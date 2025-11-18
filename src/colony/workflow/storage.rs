use super::types::{WorkflowDefinition, WorkflowRun, WorkflowRunStatus};
use crate::error::{ColonyError, ColonyResult};
use std::fs;
use std::path::{Path, PathBuf};

/// Workflow storage manager
pub struct WorkflowStorage {
    workflows_dir: PathBuf,
    runs_dir: PathBuf,
}

impl WorkflowStorage {
    /// Create a new workflow storage manager
    pub fn new(colony_root: &Path) -> Self {
        Self {
            workflows_dir: colony_root.join("workflows"),
            runs_dir: colony_root.join("workflow_runs"),
        }
    }

    /// Initialize storage directories
    pub fn initialize(&self) -> ColonyResult<()> {
        fs::create_dir_all(&self.workflows_dir).map_err(|e| {
            ColonyError::Colony(format!("Failed to create workflows directory: {}", e))
        })?;

        fs::create_dir_all(&self.runs_dir).map_err(|e| {
            ColonyError::Colony(format!("Failed to create workflow_runs directory: {}", e))
        })?;

        Ok(())
    }

    /// Save a workflow definition
    pub fn save_workflow(&self, definition: &WorkflowDefinition) -> ColonyResult<()> {
        let path = self.workflows_dir.join(format!("{}.yaml", definition.name));
        let yaml = serde_yaml::to_string(definition).map_err(|e| {
            ColonyError::Colony(format!("Failed to serialize workflow: {}", e))
        })?;

        fs::write(&path, yaml).map_err(|e| {
            ColonyError::Colony(format!("Failed to write workflow file: {}", e))
        })?;

        Ok(())
    }

    /// Load a workflow definition by name
    pub fn load_workflow(&self, name: &str) -> ColonyResult<WorkflowDefinition> {
        let path = self.workflows_dir.join(format!("{}.yaml", name));

        if !path.exists() {
            return Err(ColonyError::Colony(format!(
                "Workflow '{}' not found",
                name
            )));
        }

        let content = fs::read_to_string(&path).map_err(|e| {
            ColonyError::Colony(format!("Failed to read workflow file: {}", e))
        })?;

        let definition: WorkflowDefinition = serde_yaml::from_str(&content).map_err(|e| {
            ColonyError::Colony(format!("Failed to parse workflow YAML: {}", e))
        })?;

        Ok(definition)
    }

    /// List all workflow definitions
    pub fn list_workflows(&self) -> ColonyResult<Vec<WorkflowDefinition>> {
        if !self.workflows_dir.exists() {
            return Ok(vec![]);
        }

        let mut workflows = vec![];

        let entries = fs::read_dir(&self.workflows_dir).map_err(|e| {
            ColonyError::Colony(format!("Failed to read workflows directory: {}", e))
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(definition) = serde_yaml::from_str::<WorkflowDefinition>(&content) {
                        workflows.push(definition);
                    }
                }
            }
        }

        workflows.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(workflows)
    }

    /// Delete a workflow definition
    pub fn delete_workflow(&self, name: &str) -> ColonyResult<()> {
        let path = self.workflows_dir.join(format!("{}.yaml", name));

        if !path.exists() {
            return Err(ColonyError::Colony(format!(
                "Workflow '{}' not found",
                name
            )));
        }

        fs::remove_file(&path).map_err(|e| {
            ColonyError::Colony(format!("Failed to delete workflow: {}", e))
        })?;

        Ok(())
    }

    /// Save a workflow run
    pub fn save_run(&self, run: &WorkflowRun) -> ColonyResult<()> {
        let workflow_runs_dir = self.runs_dir.join(&run.workflow_name);
        fs::create_dir_all(&workflow_runs_dir).map_err(|e| {
            ColonyError::Colony(format!("Failed to create workflow runs directory: {}", e))
        })?;

        let path = workflow_runs_dir.join(format!("{}.json", run.id));
        let json = serde_json::to_string_pretty(run).map_err(|e| {
            ColonyError::Colony(format!("Failed to serialize workflow run: {}", e))
        })?;

        fs::write(&path, json).map_err(|e| {
            ColonyError::Colony(format!("Failed to write workflow run file: {}", e))
        })?;

        Ok(())
    }

    /// Load a workflow run by ID
    pub fn load_run(&self, run_id: &str) -> ColonyResult<WorkflowRun> {
        // Search through all workflow run directories
        if !self.runs_dir.exists() {
            return Err(ColonyError::Colony(format!(
                "Workflow run '{}' not found",
                run_id
            )));
        }

        let entries = fs::read_dir(&self.runs_dir).map_err(|e| {
            ColonyError::Colony(format!("Failed to read workflow runs directory: {}", e))
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let run_path = path.join(format!("{}.json", run_id));
                if run_path.exists() {
                    let content = fs::read_to_string(&run_path).map_err(|e| {
                        ColonyError::Colony(format!("Failed to read workflow run file: {}", e))
                    })?;

                    let run: WorkflowRun = serde_json::from_str(&content).map_err(|e| {
                        ColonyError::Colony(format!("Failed to parse workflow run JSON: {}", e))
                    })?;

                    return Ok(run);
                }
            }
        }

        Err(ColonyError::Colony(format!(
            "Workflow run '{}' not found",
            run_id
        )))
    }

    /// List all runs for a workflow
    pub fn list_runs(&self, workflow_name: &str) -> ColonyResult<Vec<WorkflowRun>> {
        let workflow_runs_dir = self.runs_dir.join(workflow_name);

        if !workflow_runs_dir.exists() {
            return Ok(vec![]);
        }

        let mut runs = vec![];

        let entries = fs::read_dir(&workflow_runs_dir).map_err(|e| {
            ColonyError::Colony(format!("Failed to read workflow runs directory: {}", e))
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(run) = serde_json::from_str::<WorkflowRun>(&content) {
                        runs.push(run);
                    }
                }
            }
        }

        // Sort by started_at (most recent first)
        runs.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        Ok(runs)
    }

    /// List all active runs (running or pending)
    pub fn list_active_runs(&self) -> ColonyResult<Vec<WorkflowRun>> {
        if !self.runs_dir.exists() {
            return Ok(vec![]);
        }

        let mut runs = vec![];

        let entries = fs::read_dir(&self.runs_dir).map_err(|e| {
            ColonyError::Colony(format!("Failed to read workflow runs directory: {}", e))
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(run_entries) = fs::read_dir(&path) {
                    for run_entry in run_entries.flatten() {
                        let run_path = run_entry.path();
                        if run_path.extension().and_then(|s| s.to_str()) == Some("json") {
                            if let Ok(content) = fs::read_to_string(&run_path) {
                                if let Ok(run) = serde_json::from_str::<WorkflowRun>(&content) {
                                    if matches!(
                                        run.status,
                                        WorkflowRunStatus::Running | WorkflowRunStatus::Pending
                                    ) {
                                        runs.push(run);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        runs.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        Ok(runs)
    }

    /// Delete a workflow run
    pub fn delete_run(&self, run_id: &str) -> ColonyResult<()> {
        // Search through all workflow run directories
        if !self.runs_dir.exists() {
            return Err(ColonyError::Colony(format!(
                "Workflow run '{}' not found",
                run_id
            )));
        }

        let entries = fs::read_dir(&self.runs_dir).map_err(|e| {
            ColonyError::Colony(format!("Failed to read workflow runs directory: {}", e))
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let run_path = path.join(format!("{}.json", run_id));
                if run_path.exists() {
                    fs::remove_file(&run_path).map_err(|e| {
                        ColonyError::Colony(format!("Failed to delete workflow run: {}", e))
                    })?;
                    return Ok(());
                }
            }
        }

        Err(ColonyError::Colony(format!(
            "Workflow run '{}' not found",
            run_id
        )))
    }
}
