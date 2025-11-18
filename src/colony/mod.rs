/// Colony - Multi-Agent Orchestration for Claude Code
///
/// This module provides commands for managing multiple Claude Code instances
/// running in parallel with proper isolation and state management.
pub mod agent;
pub mod attach;
pub mod broadcast;
pub mod config;
pub mod controller;
pub mod destroy;
pub mod executor;
pub mod health;
pub mod init;
pub mod logging;
pub mod logs;
pub mod messages_cmd;
pub mod messaging;
pub mod metrics;
pub mod metrics_cmd;
pub mod plugin;
pub mod plugin_cmd;
pub mod skills;
pub mod template;
pub mod template_cmd;
pub mod start;
pub mod state;
pub mod state_cmd;
pub mod state_integration;
pub mod status;
pub mod stop;
pub mod tasks;
pub mod tasks_cmd;
pub mod tmux;
pub mod tui;
pub mod workflow;
pub mod workflow_cmd;
pub mod worktree;

// Re-export commonly used types
pub use agent::{Agent, AgentStatus};
pub use config::ColonyConfig;
pub use controller::ColonyController;
