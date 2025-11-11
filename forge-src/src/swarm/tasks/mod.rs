use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;

pub mod board;
pub mod queue;

use crate::error::ForgeResult;

/// Task priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

/// Task status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    /// Task is available to be claimed
    Pending,
    /// Task has been claimed by an agent
    Claimed,
    /// Task is actively being worked on
    InProgress,
    /// Task is blocked by dependencies or issues
    Blocked,
    /// Task has been completed
    Completed,
    /// Task has been cancelled
    Cancelled,
}

impl TaskStatus {
    pub fn emoji(&self) -> &str {
        match self {
            TaskStatus::Pending => "⏳",
            TaskStatus::Claimed => "👤",
            TaskStatus::InProgress => "🔄",
            TaskStatus::Blocked => "🚫",
            TaskStatus::Completed => "✅",
            TaskStatus::Cancelled => "❌",
        }
    }

    pub fn display(&self) -> &str {
        match self {
            TaskStatus::Pending => "PENDING",
            TaskStatus::Claimed => "CLAIMED",
            TaskStatus::InProgress => "WORKING",
            TaskStatus::Blocked => "BLOCKED",
            TaskStatus::Completed => "DONE",
            TaskStatus::Cancelled => "CANCELLED",
        }
    }
}

/// A task in the swarm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique task ID
    pub id: String,
    /// Task title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Agent ID this task is assigned to (or "auto" for claiming)
    pub assigned_to: Option<String>,
    /// Agent ID that claimed this task
    pub claimed_by: Option<String>,
    /// Current status
    pub status: TaskStatus,
    /// Priority level
    #[serde(default)]
    pub priority: TaskPriority,
    /// Progress percentage (0-100)
    #[serde(default)]
    pub progress: u8,
    /// Task IDs that must be completed before this task
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Current blockers preventing progress
    #[serde(default)]
    pub blockers: Vec<String>,
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    /// When task was created
    pub created_at: String,
    /// When task was claimed
    pub claimed_at: Option<String>,
    /// When work started
    pub started_at: Option<String>,
    /// When task was completed
    pub completed_at: Option<String>,
    /// Last update timestamp
    pub updated_at: String,
}

impl Task {
    /// Create a new task
    pub fn new(id: String, title: String, description: String) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id,
            title,
            description,
            assigned_to: None,
            claimed_by: None,
            status: TaskStatus::Pending,
            priority: TaskPriority::Medium,
            progress: 0,
            dependencies: Vec::new(),
            blockers: Vec::new(),
            tags: Vec::new(),
            created_at: now.clone(),
            claimed_at: None,
            started_at: None,
            completed_at: None,
            updated_at: now,
        }
    }

    /// Check if task can be claimed
    pub fn can_claim(&self, agent_id: &str, completed_tasks: &HashSet<String>) -> bool {
        // Must be in pending status
        if self.status != TaskStatus::Pending {
            return false;
        }

        // Check if assigned to specific agent
        if let Some(ref assigned) = self.assigned_to {
            if assigned != "auto" && assigned != agent_id {
                return false;
            }
        }

        // Check dependencies are satisfied
        for dep in &self.dependencies {
            if !completed_tasks.contains(dep) {
                return false;
            }
        }

        true
    }

    /// Claim this task
    pub fn claim(&mut self, agent_id: &str) {
        let now = Utc::now().to_rfc3339();
        self.claimed_by = Some(agent_id.to_string());
        self.status = TaskStatus::Claimed;
        self.claimed_at = Some(now.clone());
        self.updated_at = now;
    }

    /// Start working on this task
    pub fn start(&mut self) {
        let now = Utc::now().to_rfc3339();
        self.status = TaskStatus::InProgress;
        if self.started_at.is_none() {
            self.started_at = Some(now.clone());
        }
        self.updated_at = now;
    }

    /// Update progress
    pub fn update_progress(&mut self, progress: u8) {
        self.progress = progress.min(100);
        self.updated_at = Utc::now().to_rfc3339();
        if self.status == TaskStatus::Claimed {
            self.start();
        }
    }

    /// Mark task as blocked
    pub fn block(&mut self, reason: String) {
        self.status = TaskStatus::Blocked;
        self.blockers.push(reason);
        self.updated_at = Utc::now().to_rfc3339();
    }

    /// Unblock task
    pub fn unblock(&mut self) {
        self.status = TaskStatus::InProgress;
        self.blockers.clear();
        self.updated_at = Utc::now().to_rfc3339();
    }

    /// Complete this task
    pub fn complete(&mut self) {
        let now = Utc::now().to_rfc3339();
        self.status = TaskStatus::Completed;
        self.progress = 100;
        self.completed_at = Some(now.clone());
        self.updated_at = now;
    }

    /// Cancel this task
    pub fn cancel(&mut self) {
        self.status = TaskStatus::Cancelled;
        self.updated_at = Utc::now().to_rfc3339();
    }

    /// Check if task is assigned to specific agent
    pub fn is_assigned_to(&self, agent_id: &str) -> bool {
        self.assigned_to.as_deref() == Some(agent_id)
            || self.claimed_by.as_deref() == Some(agent_id)
    }

    /// Check if task blocks other tasks
    #[allow(dead_code)]
    pub fn is_dependency_for(&self, task: &Task) -> bool {
        task.dependencies.contains(&self.id)
    }
}

/// Load a task from file
pub fn load_task(path: &Path) -> ForgeResult<Task> {
    let content = std::fs::read_to_string(path)?;
    let task = serde_json::from_str(&content)?;
    Ok(task)
}

/// Save a task to file
pub fn save_task(task: &Task, path: &Path) -> ForgeResult<()> {
    let json = serde_json::to_string_pretty(task)?;
    std::fs::write(path, json)?;
    Ok(())
}
