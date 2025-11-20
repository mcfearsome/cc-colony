//! Git-backed state backend

use crate::colony::state::{cache::StateCache, jsonl, state_config::SharedStateConfig, types::*};
use crate::error::{ColonyError, ColonyResult};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

/// Git-backed state backend
pub struct GitBackedState {
    /// State configuration
    config: SharedStateConfig,
    /// Repository root
    repo_root: PathBuf,
    /// SQLite cache
    cache: Arc<Mutex<StateCache>>,
    /// Last export time (for debouncing)
    last_export: Arc<Mutex<Option<SystemTime>>>,
}

impl GitBackedState {
    /// Create a new git-backed state instance
    pub fn new(config: SharedStateConfig, repo_root: PathBuf) -> ColonyResult<Self> {
        // Initialize state directory
        let state_dir = config.state_dir_path(&repo_root);
        std::fs::create_dir_all(&state_dir).map_err(|e| {
            ColonyError::InvalidConfig(format!("Failed to create state directory: {}", e))
        })?;

        // Initialize git repository if needed
        Self::init_git_repo(&state_dir, &config)?;

        // Initialize cache
        let cache_path = config.cache_db_path(&repo_root);
        let cache = StateCache::open(&cache_path)?;

        Ok(Self {
            config,
            repo_root,
            cache: Arc::new(Mutex::new(cache)),
            last_export: Arc::new(Mutex::new(None)),
        })
    }

    /// Initialize git repository for state
    fn init_git_repo(state_dir: &Path, config: &SharedStateConfig) -> ColonyResult<()> {
        // Check if .git directory exists
        let git_dir = state_dir.join(".git");
        if !git_dir.exists() {
            // Initialize git repo
            let output = Command::new("git")
                .args(["init"])
                .current_dir(state_dir)
                .output()
                .map_err(|e| {
                    ColonyError::InvalidConfig(format!("Failed to initialize git repo: {}", e))
                })?;

            if !output.status.success() {
                return Err(ColonyError::InvalidConfig(format!(
                    "Git init failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                )));
            }

            // Create initial commit
            Command::new("git")
                .args(["commit", "--allow-empty", "-m", "Initialize colony state"])
                .current_dir(state_dir)
                .output()
                .ok();
        }

        // If external repo is configured, set up remote
        if let Some(repo_url) = &config.repository {
            let output = Command::new("git")
                .args(["remote", "get-url", "origin"])
                .current_dir(state_dir)
                .output()
                .ok();

            let has_remote = output.is_some_and(|o| o.status.success());

            if !has_remote {
                // Add remote
                Command::new("git")
                    .args(["remote", "add", "origin", repo_url])
                    .current_dir(state_dir)
                    .output()
                    .map_err(|e| {
                        ColonyError::InvalidConfig(format!("Failed to add remote: {}", e))
                    })?;
            }
        }

        Ok(())
    }

    /// Get the state directory path
    pub fn state_dir(&self) -> PathBuf {
        self.config.state_dir_path(&self.repo_root)
    }

    /// Get the path to a schema file
    fn schema_file_path(&self, schema_name: &str) -> ColonyResult<PathBuf> {
        let schema = self.config.get_schema(schema_name).ok_or_else(|| {
            ColonyError::InvalidConfig(format!("Unknown schema: {}", schema_name))
        })?;

        Ok(self.state_dir().join(&schema.file))
    }

    /// Sync cache from JSONL file if needed
    async fn sync_cache_from_jsonl<T>(&self, schema_name: &str) -> ColonyResult<Vec<T>>
    where
        T: serde::de::DeserializeOwned,
    {
        let file_path = self.schema_file_path(schema_name)?;

        // If file doesn't exist, return empty vec
        if !file_path.exists() {
            return Ok(Vec::new());
        }

        // Check if cache needs refresh
        let file_mtime = jsonl::get_modified_time(&file_path).await?;
        let needs_refresh = {
            let cache = self.cache.lock().unwrap();
            cache.needs_refresh(schema_name, file_mtime)?
        };

        if needs_refresh {
            // Read from JSONL
            let entries: Vec<T> = jsonl::read_jsonl(&file_path).await?;

            // Update cache metadata
            {
                let cache = self.cache.lock().unwrap();
                cache.mark_synced(schema_name, file_mtime)?;
            }

            Ok(entries)
        } else {
            // Cache is up to date, but we still need to read from JSONL
            // for now (in the future we could query from cache)
            jsonl::read_jsonl(&file_path).await
        }
    }

    /// Export data to JSONL and optionally commit to git
    async fn export_to_jsonl<T>(&self, schema_name: &str, data: &[T]) -> ColonyResult<()>
    where
        T: serde::Serialize,
    {
        let file_path = self.schema_file_path(schema_name)?;

        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                ColonyError::InvalidConfig(format!("Failed to create state directory: {}", e))
            })?;
        }

        // Write to JSONL
        jsonl::write_jsonl(&file_path, data).await?;

        // Auto-commit if enabled
        if self.config.auto_commit {
            self.git_commit(schema_name).await?;
        }

        Ok(())
    }

    /// Commit changes to git
    async fn git_commit(&self, schema_name: &str) -> ColonyResult<()> {
        let state_dir = self.state_dir();

        // Add file
        let schema = self.config.get_schema(schema_name).ok_or_else(|| {
            ColonyError::InvalidConfig(format!("Unknown schema: {}", schema_name))
        })?;

        Command::new("git")
            .args(["add", &schema.file])
            .current_dir(&state_dir)
            .output()
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to git add: {}", e)))?;

        // Commit
        let commit_msg = self.config.commit_message.replace("{schema}", schema_name);
        Command::new("git")
            .args(["commit", "-m", &commit_msg])
            .current_dir(&state_dir)
            .output()
            .ok(); // Ignore errors (e.g., nothing to commit)

        // Auto-push if enabled
        if self.config.auto_push {
            Command::new("git")
                .args(["push", "origin", &self.config.branch])
                .current_dir(&state_dir)
                .output()
                .ok(); // Ignore errors (e.g., no remote)
        }

        Ok(())
    }

    /// Pull latest state from remote (if configured)
    pub async fn pull(&self) -> ColonyResult<()> {
        if self.config.repository.is_none() {
            return Ok(()); // No remote configured
        }

        let state_dir = self.state_dir();

        let output = Command::new("git")
            .args(["pull", "origin", &self.config.branch])
            .current_dir(&state_dir)
            .output()
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to git pull: {}", e)))?;

        if !output.status.success() {
            eprintln!(
                "Warning: git pull failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        Ok(())
    }

    // ========================================================================
    // Task Operations
    // ========================================================================

    /// Get all tasks
    pub async fn get_tasks(&self) -> ColonyResult<Vec<Task>> {
        // Sync cache from JSONL
        let tasks = self.sync_cache_from_jsonl::<Task>("tasks").await?;

        // Import into cache
        {
            let mut cache = self.cache.lock().unwrap();
            cache.import_tasks(&tasks)?;
        }

        // Query from cache
        let cache = self.cache.lock().unwrap();
        cache.get_tasks()
    }

    /// Get tasks by status
    pub async fn get_tasks_by_status(&self, status: TaskStatus) -> ColonyResult<Vec<Task>> {
        // Ensure cache is synced
        self.get_tasks().await?;

        // Query from cache
        let cache = self.cache.lock().unwrap();
        cache.get_tasks_by_status(status)
    }

    /// Get task by ID
    pub async fn get_task(&self, id: &str) -> ColonyResult<Option<Task>> {
        // Ensure cache is synced
        self.get_tasks().await?;

        // Query from cache
        let cache = self.cache.lock().unwrap();
        cache.get_task(id)
    }

    /// Add a new task
    pub async fn add_task(&self, task: Task) -> ColonyResult<()> {
        let mut tasks = self.get_tasks().await?;
        tasks.push(task);
        self.export_to_jsonl("tasks", &tasks).await?;

        // Update cache
        {
            let mut cache = self.cache.lock().unwrap();
            cache.import_tasks(&tasks)?;
        }

        Ok(())
    }

    /// Update a task
    pub async fn update_task(&self, updated_task: Task) -> ColonyResult<()> {
        let mut tasks = self.get_tasks().await?;

        // Find and replace the task
        if let Some(task) = tasks.iter_mut().find(|t| t.id == updated_task.id) {
            *task = updated_task;
        } else {
            return Err(ColonyError::InvalidConfig(format!(
                "Task not found: {}",
                updated_task.id
            )));
        }

        self.export_to_jsonl("tasks", &tasks).await?;

        // Update cache
        {
            let mut cache = self.cache.lock().unwrap();
            cache.import_tasks(&tasks)?;
        }

        Ok(())
    }

    /// Get ready tasks (no blockers)
    pub async fn get_ready_tasks(&self) -> ColonyResult<Vec<Task>> {
        let all_tasks = self.get_tasks().await?;

        // Get completed task IDs
        let completed_ids: Vec<String> = all_tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .map(|t| t.id.clone())
            .collect();

        // Filter to ready tasks
        let ready: Vec<Task> = all_tasks
            .into_iter()
            .filter(|t| t.is_ready(&completed_ids))
            .collect();

        Ok(ready)
    }

    // ========================================================================
    // Workflow Operations
    // ========================================================================

    /// Get all workflows
    pub async fn get_workflows(&self) -> ColonyResult<Vec<Workflow>> {
        // Sync cache from JSONL
        let workflows = self.sync_cache_from_jsonl::<Workflow>("workflows").await?;

        // Import into cache
        {
            let mut cache = self.cache.lock().unwrap();
            cache.import_workflows(&workflows)?;
        }

        // Query from cache
        let cache = self.cache.lock().unwrap();
        cache.get_workflows()
    }

    /// Get workflows by status
    pub async fn get_workflows_by_status(
        &self,
        status: WorkflowStatus,
    ) -> ColonyResult<Vec<Workflow>> {
        // Ensure cache is synced
        self.get_workflows().await?;

        // Query from cache
        let cache = self.cache.lock().unwrap();
        cache.get_workflows_by_status(status)
    }

    /// Get workflow by ID
    pub async fn get_workflow(&self, id: &str) -> ColonyResult<Option<Workflow>> {
        // Ensure cache is synced
        self.get_workflows().await?;

        // Query from cache
        let cache = self.cache.lock().unwrap();
        cache.get_workflow(id)
    }

    /// Add a new workflow
    pub async fn add_workflow(&self, workflow: Workflow) -> ColonyResult<()> {
        let mut workflows = self.get_workflows().await?;
        workflows.push(workflow);
        self.export_to_jsonl("workflows", &workflows).await?;

        // Update cache
        {
            let mut cache = self.cache.lock().unwrap();
            cache.import_workflows(&workflows)?;
        }

        Ok(())
    }

    /// Update a workflow
    pub async fn update_workflow(&self, updated_workflow: Workflow) -> ColonyResult<()> {
        let mut workflows = self.get_workflows().await?;

        // Find and replace the workflow
        if let Some(workflow) = workflows.iter_mut().find(|w| w.id == updated_workflow.id) {
            *workflow = updated_workflow;
        } else {
            return Err(ColonyError::InvalidConfig(format!(
                "Workflow not found: {}",
                updated_workflow.id
            )));
        }

        self.export_to_jsonl("workflows", &workflows).await?;

        // Update cache
        {
            let mut cache = self.cache.lock().unwrap();
            cache.import_workflows(&workflows)?;
        }

        Ok(())
    }

    // ========================================================================
    // Memory Operations
    // ========================================================================

    /// Get all memory entries
    pub async fn get_memory(&self) -> ColonyResult<Vec<MemoryEntry>> {
        // For now, we don't have a "memory" schema in the default config
        // This would need to be added to state_config.rs default_schemas()
        // For now, return empty vec
        Ok(Vec::new())
    }

    /// Add a memory entry
    pub async fn add_memory(&self, entry: MemoryEntry) -> ColonyResult<()> {
        let mut entries = self.get_memory().await?;
        entries.push(entry);
        // Would need "memory" schema defined
        // self.export_to_jsonl("memory", &entries).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_git_backed_state_init() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path().to_path_buf();

        let config = SharedStateConfig::default();
        let state = GitBackedState::new(config, repo_root.clone()).unwrap();

        // Check that state directory was created
        let state_dir = state.state_dir();
        assert!(state_dir.exists());

        // Check that git repo was initialized
        let git_dir = state_dir.join(".git");
        assert!(git_dir.exists());
    }

    #[tokio::test]
    async fn test_task_operations() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path().to_path_buf();

        let config = SharedStateConfig::default();
        let state = GitBackedState::new(config, repo_root).unwrap();

        // Create a test task
        let task = Task::new("Test Task".to_string());
        let task_id = task.id.clone();

        // Add task
        state.add_task(task).await.unwrap();

        // Get tasks
        let tasks = state.get_tasks().await.unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Test Task");

        // Get task by ID
        let retrieved = state.get_task(&task_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title, "Test Task");

        // Update task
        let mut updated_task = state.get_task(&task_id).await.unwrap().unwrap();
        updated_task.status = TaskStatus::InProgress;
        state.update_task(updated_task).await.unwrap();

        // Verify update
        let task = state.get_task(&task_id).await.unwrap().unwrap();
        assert_eq!(task.status, TaskStatus::InProgress);
    }

    #[tokio::test]
    async fn test_workflow_operations() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path().to_path_buf();

        let config = SharedStateConfig::default();
        let state = GitBackedState::new(config, repo_root).unwrap();

        // Create a test workflow
        let workflow = Workflow::new("Test Workflow".to_string());
        let workflow_id = workflow.id.clone();

        // Add workflow
        state.add_workflow(workflow).await.unwrap();

        // Get workflows
        let workflows = state.get_workflows().await.unwrap();
        assert_eq!(workflows.len(), 1);
        assert_eq!(workflows[0].name, "Test Workflow");

        // Get workflow by ID
        let retrieved = state.get_workflow(&workflow_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Workflow");
    }

    #[tokio::test]
    async fn test_ready_tasks() {
        let dir = tempdir().unwrap();
        let repo_root = dir.path().to_path_buf();

        let config = SharedStateConfig::default();
        let state = GitBackedState::new(config, repo_root).unwrap();

        // Create tasks with dependencies
        let task1 = Task::new("Task 1".to_string());
        let task1_id = task1.id.clone();

        let task2 = Task::new_with_blockers("Task 2".to_string(), vec![task1_id.clone()]);

        state.add_task(task1).await.unwrap();
        state.add_task(task2).await.unwrap();

        // Task 2 should not be ready (blocked by task 1)
        let ready = state.get_ready_tasks().await.unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].title, "Task 1");

        // Complete task 1
        let mut task1 = state.get_task(&task1_id).await.unwrap().unwrap();
        task1.status = TaskStatus::Completed;
        task1.completed = Some(Utc::now());
        state.update_task(task1).await.unwrap();

        // Now task 2 should be ready
        let ready = state.get_ready_tasks().await.unwrap();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].title, "Task 2");
    }
}
