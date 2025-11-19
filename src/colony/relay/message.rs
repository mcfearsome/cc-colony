use serde::{Deserialize, Serialize};

/// Messages sent from Colony CLI to Relay Service
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CliMessage {
    /// Initial connection with authentication
    Connect {
        colony_id: String,
        auth_token: String,
        version: String,
    },
    /// State update with current colony status
    StateUpdate {
        colony_id: String,
        timestamp: i64,
        agents: Vec<AgentState>,
        tasks: Vec<TaskState>,
        messages: Vec<MessageState>,
    },
    /// Result of a command execution
    CommandResult {
        request_id: String,
        success: bool,
        output: Option<String>,
        error: Option<String>,
    },
    /// Heartbeat to keep connection alive
    Pong,
}

/// Messages sent from Web Client to Relay Service
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Subscribe to a colony
    Subscribe {
        colony_id: String,
        auth_token: String,
    },
    /// Execute a command on the colony
    Command {
        request_id: String,
        colony_id: String,
        command: Command,
    },
}

/// Messages sent from Relay Service to Colony CLI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RelayToCliMessage {
    /// Command to execute
    Command {
        request_id: String,
        command: Command,
    },
    /// Heartbeat ping
    Ping,
    /// Connection accepted
    Connected {
        colony_id: String,
    },
    /// Error occurred
    Error {
        message: String,
    },
}

/// Messages sent from Relay Service to Web Client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RelayToClientMessage {
    /// State update
    StateUpdate {
        timestamp: i64,
        agents: Vec<AgentState>,
        tasks: Vec<TaskState>,
        messages: Vec<MessageState>,
    },
    /// Command execution result
    CommandResult {
        request_id: String,
        success: bool,
        output: Option<String>,
        error: Option<String>,
    },
    /// Subscription confirmed
    Subscribed {
        colony_id: String,
    },
    /// Error occurred
    Error {
        message: String,
    },
}

/// Command to execute on the colony
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum Command {
    /// Send a message to an agent
    SendMessage {
        to: String,
        content: String,
        message_type: String,
    },
    /// Broadcast a message to all agents
    BroadcastMessage {
        content: String,
    },
    /// Create a new task
    CreateTask {
        title: String,
        description: String,
        assigned_to: Option<String>,
        priority: Option<String>,
    },
    /// Stop an agent
    StopAgent {
        agent_id: String,
    },
    /// Start an agent
    StartAgent {
        agent_id: String,
    },
    /// Restart an agent
    RestartAgent {
        agent_id: String,
    },
}

/// Agent state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub id: String,
    pub role: String,
    pub status: AgentStatus,
    pub last_activity: Option<i64>,
}

/// Agent status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Running,
    Idle,
    Failed,
    Stopped,
}

/// Task state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskState {
    pub id: String,
    pub title: String,
    pub status: TaskStatus,
    pub assigned_to: Option<String>,
    pub priority: String,
    pub created_at: String,
}

/// Task status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Claimed,
    InProgress,
    Blocked,
    Completed,
    Cancelled,
}

/// Message state snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageState {
    pub id: String,
    pub from: String,
    pub to: String,
    pub content: String,
    pub timestamp: String,
    pub message_type: String,
}
