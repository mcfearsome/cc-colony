//! Data types for colony state

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Task status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    /// Task is ready to be worked on (no blockers)
    Ready,
    /// Task is blocked by other tasks
    Blocked,
    /// Task is currently being worked on
    InProgress,
    /// Task has been completed
    Completed,
    /// Task has been cancelled
    Cancelled,
}

/// A task in the colony task queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task ID (e.g., "task-a1b2c3")
    pub id: String,
    /// Task title
    pub title: String,
    /// Optional detailed description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Current status
    pub status: TaskStatus,
    /// When the task was created
    pub created: DateTime<Utc>,
    /// Agent ID that claimed this task
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned: Option<String>,
    /// Task IDs that block this task
    #[serde(default)]
    pub blockers: Vec<String>,
    /// When the task was completed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<DateTime<Utc>>,
    /// Additional metadata
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl Task {
    /// Create a new task
    pub fn new(title: String) -> Self {
        let id = TaskIdGenerator::generate("task");
        Self {
            id,
            title,
            description: None,
            status: TaskStatus::Ready,
            created: Utc::now(),
            assigned: None,
            blockers: Vec::new(),
            completed: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Create a new task with blockers
    pub fn new_with_blockers(title: String, blockers: Vec<String>) -> Self {
        let mut task = Self::new(title);
        let is_empty = blockers.is_empty();
        task.blockers = blockers;
        task.status = if is_empty {
            TaskStatus::Ready
        } else {
            TaskStatus::Blocked
        };
        task
    }

    /// Check if task is ready (no open blockers)
    pub fn is_ready(&self, completed_tasks: &[String]) -> bool {
        if !matches!(self.status, TaskStatus::Blocked | TaskStatus::Ready) {
            return false;
        }

        self.blockers
            .iter()
            .all(|blocker| completed_tasks.contains(blocker))
    }
}

/// Workflow status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowStatus {
    /// Workflow is pending (not started)
    Pending,
    /// Workflow is currently running
    Running,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed
    Failed,
    /// Workflow was cancelled
    Cancelled,
}

/// Status of a workflow step
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    /// Step is pending
    Pending,
    /// Step is running
    Running,
    /// Step completed successfully
    Completed,
    /// Step failed
    Failed,
    /// Step was skipped
    Skipped,
}

/// Information about a workflow step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// Step name
    pub name: String,
    /// Step status
    pub status: StepStatus,
    /// When the step started
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started: Option<DateTime<Utc>>,
    /// When the step completed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<DateTime<Utc>>,
    /// Agent that executed this step
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    /// Step output (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,
    /// Error message (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// A workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Unique workflow ID (e.g., "wf-a1b2c3")
    pub id: String,
    /// Workflow name
    pub name: String,
    /// Current status
    pub status: WorkflowStatus,
    /// When the workflow started
    pub started: DateTime<Utc>,
    /// When the workflow completed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<DateTime<Utc>>,
    /// Current step being executed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_step: Option<String>,
    /// Map of step name to step info
    #[serde(default)]
    pub steps: std::collections::HashMap<String, WorkflowStep>,
    /// Workflow input
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
    /// Workflow output
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,
}

impl Workflow {
    /// Create a new workflow
    pub fn new(name: String) -> Self {
        let id = TaskIdGenerator::generate("wf");
        Self {
            id,
            name,
            status: WorkflowStatus::Pending,
            started: Utc::now(),
            completed: None,
            current_step: None,
            steps: std::collections::HashMap::new(),
            input: None,
            output: None,
        }
    }
}

/// Agent memory entry type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MemoryType {
    /// Context information
    Context,
    /// Something learned
    Learned,
    /// A todo item
    Todo,
    /// A note
    Note,
}

/// Agent memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// When this was recorded
    pub timestamp: DateTime<Utc>,
    /// Type of memory
    #[serde(rename = "type")]
    pub entry_type: MemoryType,
    /// Optional key (for context entries)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// Optional value (for context entries)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Optional content (for learned/todo/note entries)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// Hash-based ID generator (Beads-style)
pub struct TaskIdGenerator;

impl TaskIdGenerator {
    /// Generate a collision-free ID with the given prefix
    pub fn generate(prefix: &str) -> String {
        Self::generate_with_length(prefix, 6)
    }

    /// Generate an ID with specific length
    pub fn generate_with_length(prefix: &str, length: usize) -> String {
        let mut hasher = Sha256::new();

        // Add timestamp
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        hasher.update(now.to_le_bytes());

        // Add random component
        let random: u64 = rand::random();
        hasher.update(random.to_le_bytes());

        // Hash
        let hash = hasher.finalize();
        let hex = hex::encode(&hash[..length / 2]);

        format!("{}-{}", prefix, hex)
    }

    /// Generate a child ID (e.g., "task-a1b2c3.1")
    pub fn child_id(parent_id: &str, index: usize) -> String {
        format!("{}.{}", parent_id, index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id() {
        let id1 = TaskIdGenerator::generate("task");
        let id2 = TaskIdGenerator::generate("task");

        // IDs should be different
        assert_ne!(id1, id2);

        // IDs should have correct format
        assert!(id1.starts_with("task-"));
        assert_eq!(id1.len(), "task-".len() + 6);
    }

    #[test]
    fn test_child_id() {
        let parent = "task-abc123";
        let child = TaskIdGenerator::child_id(parent, 1);

        assert_eq!(child, "task-abc123.1");
    }

    #[test]
    fn test_task_is_ready() {
        let mut task = Task::new("Test".to_string());
        task.blockers = vec!["task-1".to_string(), "task-2".to_string()];
        task.status = TaskStatus::Blocked;

        // Not ready - blockers not completed
        assert!(!task.is_ready(&[]));

        // Not ready - only one blocker completed
        assert!(!task.is_ready(&["task-1".to_string()]));

        // Ready - all blockers completed
        assert!(task.is_ready(&["task-1".to_string(), "task-2".to_string()]));
    }
}
