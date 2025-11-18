//! SQLite cache layer for fast state queries

use crate::error::{ColonyError, ColonyResult};
use crate::colony::state::types::{MemoryEntry, Task, TaskStatus, Workflow, WorkflowStatus};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::time::SystemTime;

/// SQLite cache for state queries
pub struct StateCache {
    conn: Connection,
}

impl StateCache {
    /// Open or create a cache database
    pub fn open(path: &Path) -> ColonyResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                ColonyError::InvalidConfig(format!("Failed to create cache directory: {}", e))
            })?;
        }

        let conn = Connection::open(path)
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to open cache database: {}", e)))?;

        let cache = Self { conn };
        cache.initialize_schema()?;
        Ok(cache)
    }

    /// Initialize database schema
    fn initialize_schema(&self) -> ColonyResult<()> {
        self.conn
            .execute_batch(
                r#"
                -- Tasks table
                CREATE TABLE IF NOT EXISTS tasks (
                    id TEXT PRIMARY KEY,
                    title TEXT NOT NULL,
                    description TEXT,
                    status TEXT NOT NULL,
                    created TEXT NOT NULL,
                    assigned TEXT,
                    blockers TEXT,  -- JSON array
                    completed TEXT,
                    metadata TEXT   -- JSON object
                );

                CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
                CREATE INDEX IF NOT EXISTS idx_tasks_assigned ON tasks(assigned);
                CREATE INDEX IF NOT EXISTS idx_tasks_created ON tasks(created);

                -- Workflows table
                CREATE TABLE IF NOT EXISTS workflows (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    status TEXT NOT NULL,
                    started TEXT NOT NULL,
                    completed TEXT,
                    current_step TEXT,
                    steps TEXT,     -- JSON object (map of step name to step info)
                    input TEXT,     -- JSON object
                    output TEXT     -- JSON object
                );

                CREATE INDEX IF NOT EXISTS idx_workflows_status ON workflows(status);
                CREATE INDEX IF NOT EXISTS idx_workflows_started ON workflows(started);

                -- Memory entries table
                CREATE TABLE IF NOT EXISTS memory (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    timestamp TEXT NOT NULL,
                    type TEXT NOT NULL,
                    key TEXT,
                    value TEXT,
                    content TEXT
                );

                CREATE INDEX IF NOT EXISTS idx_memory_timestamp ON memory(timestamp);
                CREATE INDEX IF NOT EXISTS idx_memory_type ON memory(type);
                CREATE INDEX IF NOT EXISTS idx_memory_key ON memory(key);

                -- Metadata table for tracking JSONL file modification times
                CREATE TABLE IF NOT EXISTS cache_metadata (
                    schema_name TEXT PRIMARY KEY,
                    last_synced INTEGER NOT NULL  -- Unix timestamp in nanoseconds
                );
                "#,
            )
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to create schema: {}", e)))?;

        Ok(())
    }

    /// Check if cache needs refresh for a schema
    pub fn needs_refresh(&self, schema_name: &str, file_mtime: SystemTime) -> ColonyResult<bool> {
        let file_nanos = file_mtime
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64;

        let cached_nanos: Option<i64> = self
            .conn
            .query_row(
                "SELECT last_synced FROM cache_metadata WHERE schema_name = ?",
                params![schema_name],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to check cache metadata: {}", e)))?;

        Ok(cached_nanos.map_or(true, |cached| file_nanos > cached))
    }

    /// Mark cache as synced for a schema
    pub fn mark_synced(&self, schema_name: &str, file_mtime: SystemTime) -> ColonyResult<()> {
        let file_nanos = file_mtime
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64;

        self.conn
            .execute(
                "INSERT OR REPLACE INTO cache_metadata (schema_name, last_synced) VALUES (?, ?)",
                params![schema_name, file_nanos],
            )
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to update cache metadata: {}", e)))?;

        Ok(())
    }

    // ========================================================================
    // Task Operations
    // ========================================================================

    /// Import tasks from JSONL into cache
    pub fn import_tasks(&mut self, tasks: &[Task]) -> ColonyResult<()> {
        let tx = self
            .conn
            .transaction()
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to start transaction: {}", e)))?;

        // Clear existing tasks
        tx.execute("DELETE FROM tasks", [])
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to clear tasks: {}", e)))?;

        // Insert all tasks
        for task in tasks {
            tx.execute(
                r#"
                INSERT INTO tasks (id, title, description, status, created, assigned, blockers, completed, metadata)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                params![
                    task.id,
                    task.title,
                    task.description,
                    serde_json::to_string(&task.status).unwrap(),
                    task.created.to_rfc3339(),
                    task.assigned,
                    serde_json::to_string(&task.blockers).unwrap(),
                    task.completed.map(|dt| dt.to_rfc3339()),
                    serde_json::to_string(&task.metadata).unwrap(),
                ],
            )
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to insert task: {}", e)))?;
        }

        tx.commit()
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to commit transaction: {}", e)))?;

        Ok(())
    }

    /// Get all tasks
    pub fn get_tasks(&self) -> ColonyResult<Vec<Task>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, description, status, created, assigned, blockers, completed, metadata FROM tasks ORDER BY created DESC")
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to prepare query: {}", e)))?;

        let tasks = stmt
            .query_map([], |row| {
                Ok(Task {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    status: serde_json::from_str(&row.get::<_, String>(3)?).unwrap(),
                    created: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    assigned: row.get(5)?,
                    blockers: serde_json::from_str(&row.get::<_, String>(6)?).unwrap(),
                    completed: row
                        .get::<_, Option<String>>(7)?
                        .map(|s: String| chrono::DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&chrono::Utc)),
                    metadata: serde_json::from_str(&row.get::<_, String>(8)?).unwrap(),
                })
            })
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to query tasks: {}", e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to parse task row: {}", e)))?;

        Ok(tasks)
    }

    /// Get tasks by status
    pub fn get_tasks_by_status(&self, status: TaskStatus) -> ColonyResult<Vec<Task>> {
        let status_str = serde_json::to_string(&status).unwrap();
        let mut stmt = self
            .conn
            .prepare("SELECT id, title, description, status, created, assigned, blockers, completed, metadata FROM tasks WHERE status = ? ORDER BY created DESC")
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to prepare query: {}", e)))?;

        let tasks = stmt
            .query_map(params![status_str], |row| {
                Ok(Task {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    description: row.get(2)?,
                    status: serde_json::from_str(&row.get::<_, String>(3)?).unwrap(),
                    created: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    assigned: row.get(5)?,
                    blockers: serde_json::from_str(&row.get::<_, String>(6)?).unwrap(),
                    completed: row
                        .get::<_, Option<String>>(7)?
                        .map(|s: String| chrono::DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&chrono::Utc)),
                    metadata: serde_json::from_str(&row.get::<_, String>(8)?).unwrap(),
                })
            })
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to query tasks: {}", e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to parse task row: {}", e)))?;

        Ok(tasks)
    }

    /// Get task by ID
    pub fn get_task(&self, id: &str) -> ColonyResult<Option<Task>> {
        self.conn
            .query_row(
                "SELECT id, title, description, status, created, assigned, blockers, completed, metadata FROM tasks WHERE id = ?",
                params![id],
                |row| {
                    Ok(Task {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        description: row.get(2)?,
                        status: serde_json::from_str(&row.get::<_, String>(3)?).unwrap(),
                        created: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                            .unwrap()
                            .with_timezone(&chrono::Utc),
                        assigned: row.get(5)?,
                        blockers: serde_json::from_str(&row.get::<_, String>(6)?).unwrap(),
                        completed: row
                            .get::<_, Option<String>>(7)?
                            .map(|s: String| chrono::DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&chrono::Utc)),
                        metadata: serde_json::from_str(&row.get::<_, String>(8)?).unwrap(),
                    })
                },
            )
            .optional()
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to query task: {}", e)))
    }

    // ========================================================================
    // Workflow Operations
    // ========================================================================

    /// Import workflows from JSONL into cache
    pub fn import_workflows(&mut self, workflows: &[Workflow]) -> ColonyResult<()> {
        let tx = self
            .conn
            .transaction()
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to start transaction: {}", e)))?;

        // Clear existing workflows
        tx.execute("DELETE FROM workflows", [])
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to clear workflows: {}", e)))?;

        // Insert all workflows
        for workflow in workflows {
            tx.execute(
                r#"
                INSERT INTO workflows (id, name, status, started, completed, current_step, steps, input, output)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
                params![
                    workflow.id,
                    workflow.name,
                    serde_json::to_string(&workflow.status).unwrap(),
                    workflow.started.to_rfc3339(),
                    workflow.completed.map(|dt| dt.to_rfc3339()),
                    workflow.current_step,
                    serde_json::to_string(&workflow.steps).unwrap(),
                    workflow.input.as_ref().map(|v| serde_json::to_string(v).unwrap()),
                    workflow.output.as_ref().map(|v| serde_json::to_string(v).unwrap()),
                ],
            )
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to insert workflow: {}", e)))?;
        }

        tx.commit()
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to commit transaction: {}", e)))?;

        Ok(())
    }

    /// Get all workflows
    pub fn get_workflows(&self) -> ColonyResult<Vec<Workflow>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, status, started, completed, current_step, steps, input, output FROM workflows ORDER BY started DESC")
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to prepare query: {}", e)))?;

        let workflows = stmt
            .query_map([], |row| {
                Ok(Workflow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    status: serde_json::from_str(&row.get::<_, String>(2)?).unwrap(),
                    started: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    completed: row
                        .get::<_, Option<String>>(4)?
                        .map(|s: String| chrono::DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&chrono::Utc)),
                    current_step: row.get(5)?,
                    steps: serde_json::from_str(&row.get::<_, String>(6)?).unwrap(),
                    input: row
                        .get::<_, Option<String>>(7)?
                        .map(|s: String| serde_json::from_str(&s).unwrap()),
                    output: row
                        .get::<_, Option<String>>(8)?
                        .map(|s: String| serde_json::from_str(&s).unwrap()),
                })
            })
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to query workflows: {}", e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to parse workflow row: {}", e)))?;

        Ok(workflows)
    }

    /// Get workflows by status
    pub fn get_workflows_by_status(&self, status: WorkflowStatus) -> ColonyResult<Vec<Workflow>> {
        let status_str = serde_json::to_string(&status).unwrap();
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, status, started, completed, current_step, steps, input, output FROM workflows WHERE status = ? ORDER BY started DESC")
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to prepare query: {}", e)))?;

        let workflows = stmt
            .query_map(params![status_str], |row| {
                Ok(Workflow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    status: serde_json::from_str(&row.get::<_, String>(2)?).unwrap(),
                    started: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    completed: row
                        .get::<_, Option<String>>(4)?
                        .map(|s: String| chrono::DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&chrono::Utc)),
                    current_step: row.get(5)?,
                    steps: serde_json::from_str(&row.get::<_, String>(6)?).unwrap(),
                    input: row
                        .get::<_, Option<String>>(7)?
                        .map(|s: String| serde_json::from_str(&s).unwrap()),
                    output: row
                        .get::<_, Option<String>>(8)?
                        .map(|s: String| serde_json::from_str(&s).unwrap()),
                })
            })
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to query workflows: {}", e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to parse workflow row: {}", e)))?;

        Ok(workflows)
    }

    /// Get workflow by ID
    pub fn get_workflow(&self, id: &str) -> ColonyResult<Option<Workflow>> {
        self.conn
            .query_row(
                "SELECT id, name, status, started, completed, current_step, steps, input, output FROM workflows WHERE id = ?",
                params![id],
                |row| {
                    Ok(Workflow {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        status: serde_json::from_str(&row.get::<_, String>(2)?).unwrap(),
                        started: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                            .unwrap()
                            .with_timezone(&chrono::Utc),
                        completed: row
                            .get::<_, Option<String>>(4)?
                            .map(|s: String| chrono::DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&chrono::Utc)),
                        current_step: row.get(5)?,
                        steps: serde_json::from_str(&row.get::<_, String>(6)?).unwrap(),
                        input: row
                            .get::<_, Option<String>>(7)?
                            .map(|s: String| serde_json::from_str(&s).unwrap()),
                        output: row
                            .get::<_, Option<String>>(8)?
                            .map(|s: String| serde_json::from_str(&s).unwrap()),
                    })
                },
            )
            .optional()
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to query workflow: {}", e)))
    }

    // ========================================================================
    // Memory Operations
    // ========================================================================

    /// Import memory entries from JSONL into cache
    pub fn import_memory(&mut self, entries: &[MemoryEntry]) -> ColonyResult<()> {
        let tx = self
            .conn
            .transaction()
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to start transaction: {}", e)))?;

        // Clear existing memory
        tx.execute("DELETE FROM memory", [])
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to clear memory: {}", e)))?;

        // Insert all entries
        for entry in entries {
            tx.execute(
                r#"
                INSERT INTO memory (timestamp, type, key, value, content)
                VALUES (?, ?, ?, ?, ?)
                "#,
                params![
                    entry.timestamp.to_rfc3339(),
                    serde_json::to_string(&entry.entry_type).unwrap(),
                    entry.key,
                    entry.value,
                    entry.content,
                ],
            )
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to insert memory entry: {}", e)))?;
        }

        tx.commit()
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to commit transaction: {}", e)))?;

        Ok(())
    }

    /// Get all memory entries
    pub fn get_memory(&self) -> ColonyResult<Vec<MemoryEntry>> {
        let mut stmt = self
            .conn
            .prepare("SELECT timestamp, type, key, value, content FROM memory ORDER BY timestamp DESC")
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to prepare query: {}", e)))?;

        let entries = stmt
            .query_map([], |row| {
                Ok(MemoryEntry {
                    timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(0)?)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    entry_type: serde_json::from_str(&row.get::<_, String>(1)?).unwrap(),
                    key: row.get(2)?,
                    value: row.get(3)?,
                    content: row.get(4)?,
                })
            })
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to query memory: {}", e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ColonyError::InvalidConfig(format!("Failed to parse memory row: {}", e)))?;

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colony::state::types::{MemoryType, StepStatus, WorkflowStep};
    use chrono::Utc;
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn test_cache_tasks() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let mut cache = StateCache::open(&db_path).unwrap();

        // Create test tasks
        let task1 = Task {
            id: "task-001".to_string(),
            title: "Test Task 1".to_string(),
            description: Some("Description 1".to_string()),
            status: TaskStatus::Ready,
            created: Utc::now(),
            assigned: None,
            blockers: vec![],
            completed: None,
            metadata: serde_json::Value::Null,
        };

        let task2 = Task {
            id: "task-002".to_string(),
            title: "Test Task 2".to_string(),
            description: None,
            status: TaskStatus::InProgress,
            created: Utc::now(),
            assigned: Some("agent-1".to_string()),
            blockers: vec!["task-001".to_string()],
            completed: None,
            metadata: serde_json::json!({"priority": "high"}),
        };

        // Import tasks
        cache.import_tasks(&[task1.clone(), task2.clone()]).unwrap();

        // Get all tasks
        let all_tasks = cache.get_tasks().unwrap();
        assert_eq!(all_tasks.len(), 2);

        // Get tasks by status
        let ready_tasks = cache.get_tasks_by_status(TaskStatus::Ready).unwrap();
        assert_eq!(ready_tasks.len(), 1);
        assert_eq!(ready_tasks[0].id, "task-001");

        // Get task by ID
        let task = cache.get_task("task-002").unwrap();
        assert!(task.is_some());
        assert_eq!(task.unwrap().title, "Test Task 2");
    }

    #[test]
    fn test_cache_workflows() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let mut cache = StateCache::open(&db_path).unwrap();

        // Create test workflow
        let mut steps = HashMap::new();
        steps.insert(
            "step1".to_string(),
            WorkflowStep {
                name: "Step 1".to_string(),
                status: StepStatus::Completed,
                started: Some(Utc::now()),
                completed: Some(Utc::now()),
                agent: Some("agent-1".to_string()),
                output: None,
                error: None,
            },
        );

        let workflow = Workflow {
            id: "wf-001".to_string(),
            name: "Test Workflow".to_string(),
            status: WorkflowStatus::Running,
            started: Utc::now(),
            completed: None,
            current_step: Some("step1".to_string()),
            steps,
            input: Some(serde_json::json!({"param": "value"})),
            output: None,
        };

        // Import workflow
        cache.import_workflows(&[workflow.clone()]).unwrap();

        // Get all workflows
        let all_workflows = cache.get_workflows().unwrap();
        assert_eq!(all_workflows.len(), 1);

        // Get workflows by status
        let running_workflows = cache.get_workflows_by_status(WorkflowStatus::Running).unwrap();
        assert_eq!(running_workflows.len(), 1);

        // Get workflow by ID
        let wf = cache.get_workflow("wf-001").unwrap();
        assert!(wf.is_some());
        assert_eq!(wf.unwrap().name, "Test Workflow");
    }

    #[test]
    fn test_cache_memory() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let mut cache = StateCache::open(&db_path).unwrap();

        // Create test memory entries
        let entry1 = MemoryEntry {
            timestamp: Utc::now(),
            entry_type: MemoryType::Context,
            key: Some("api_url".to_string()),
            value: Some("https://api.example.com".to_string()),
            content: None,
        };

        let entry2 = MemoryEntry {
            timestamp: Utc::now(),
            entry_type: MemoryType::Learned,
            key: None,
            value: None,
            content: Some("The API rate limit is 100 requests per hour".to_string()),
        };

        // Import memory
        cache.import_memory(&[entry1, entry2]).unwrap();

        // Get all memory
        let all_memory = cache.get_memory().unwrap();
        assert_eq!(all_memory.len(), 2);
    }

    #[test]
    fn test_cache_metadata() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let cache = StateCache::open(&db_path).unwrap();

        let now = SystemTime::now();

        // Initially should need refresh
        assert!(cache.needs_refresh("tasks", now).unwrap());

        // Mark as synced
        cache.mark_synced("tasks", now).unwrap();

        // Should not need refresh with same time
        assert!(!cache.needs_refresh("tasks", now).unwrap());

        // Should need refresh with newer time
        let later = SystemTime::now();
        assert!(cache.needs_refresh("tasks", later).unwrap());
    }
}
