/// Forge Colony - Multi-Agent Orchestration for Claude Code
///
/// This module provides commands for managing multiple Claude Code instances
/// running in parallel with proper isolation and state management.
pub mod agent;
pub mod attach;
pub mod broadcast;
pub mod config;
pub mod controller;
pub mod destroy;
pub mod init;
pub mod logs;
pub mod messages_cmd;
pub mod messaging;
pub mod start;
pub mod status;
pub mod stop;
pub mod tasks;
pub mod tasks_cmd;
pub mod tmux;
pub mod tui;
pub mod worktree;

// Re-export commonly used types
pub use agent::{Agent, AgentStatus};
pub use config::ColonyConfig;
pub use controller::ColonyController;
