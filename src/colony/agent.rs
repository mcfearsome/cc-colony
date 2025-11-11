use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Child;

use crate::colony::config::AgentConfig;

/// Status of an agent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    /// Agent is idle/not started
    Idle,
    /// Agent is currently running
    Running,
    /// Agent completed successfully
    Completed,
    /// Agent failed or crashed
    Failed,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentStatus::Idle => write!(f, "idle"),
            AgentStatus::Running => write!(f, "running"),
            AgentStatus::Completed => write!(f, "completed"),
            AgentStatus::Failed => write!(f, "failed"),
        }
    }
}

/// Represents a single Claude Code agent instance
pub struct Agent {
    /// Agent configuration
    pub config: AgentConfig,
    /// Path to the agent's worktree
    pub worktree_path: PathBuf,
    /// Path to the agent's Claude project directory
    pub project_path: PathBuf,
    /// Path to the agent's log file
    pub log_path: PathBuf,
    /// The running Claude Code process (if started)
    pub process: Option<Child>,
    /// Current status
    pub status: AgentStatus,
    /// Process ID (if running)
    pub pid: Option<u32>,
}

impl Agent {
    /// Create a new agent
    pub fn new(
        config: AgentConfig,
        worktree_path: PathBuf,
        project_path: PathBuf,
        log_path: PathBuf,
    ) -> Self {
        Self {
            config,
            worktree_path,
            project_path,
            log_path,
            process: None,
            status: AgentStatus::Idle,
            pid: None,
        }
    }

    /// Get the agent's ID
    pub fn id(&self) -> &str {
        &self.config.id
    }

    /// Check if the agent is running
    pub fn is_running(&self) -> bool {
        self.status == AgentStatus::Running
    }

    /// Update the agent's status
    pub fn set_status(&mut self, status: AgentStatus) {
        self.status = status;
    }
}

/// Persistent state for tracking agents across CLI invocations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub id: String,
    pub status: AgentStatus,
    pub pid: Option<u32>,
    pub worktree_path: PathBuf,
    pub project_path: PathBuf,
    pub log_path: PathBuf,
}

impl From<&Agent> for AgentState {
    fn from(agent: &Agent) -> Self {
        Self {
            id: agent.config.id.clone(),
            status: agent.status.clone(),
            pid: agent.pid,
            worktree_path: agent.worktree_path.clone(),
            project_path: agent.project_path.clone(),
            log_path: agent.log_path.clone(),
        }
    }
}
