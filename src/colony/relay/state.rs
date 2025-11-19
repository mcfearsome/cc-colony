use crate::colony::controller::ColonyController;
use crate::colony::messaging;
use crate::colony::tasks::queue::TaskQueue;
use crate::error::ColonyResult;
use chrono::Utc;
use std::path::Path;

use super::message::{AgentState, AgentStatus, MessageState, TaskState, TaskStatus};

/// Gather current colony state for synchronization
pub async fn gather_colony_state(
    colony_root: &Path,
    controller: &ColonyController,
) -> ColonyResult<(Vec<AgentState>, Vec<TaskState>, Vec<MessageState>)> {
    // Gather agent states
    let agents = gather_agent_states(controller).await?;

    // Gather task states
    let tasks = gather_task_states(colony_root)?;

    // Gather recent messages (last 50)
    let messages = gather_message_states(colony_root)?;

    Ok((agents, tasks, messages))
}

/// Gather agent states from tmux
async fn gather_agent_states(controller: &ColonyController) -> ColonyResult<Vec<AgentState>> {
    let mut agent_states = Vec::new();

    for agent in &controller.config().agents {
        let status = check_agent_status(&controller.config().session_name(), &agent.id).await;

        agent_states.push(AgentState {
            id: agent.id.clone(),
            role: agent.role.clone(),
            status,
            last_activity: Some(Utc::now().timestamp()),
        });
    }

    Ok(agent_states)
}

/// Check if an agent tmux pane is running
async fn check_agent_status(session_name: &str, agent_id: &str) -> AgentStatus {
    // Try to get pane info from tmux
    let output = tokio::process::Command::new("tmux")
        .args([
            "list-panes",
            "-t",
            &format!("{}:{}", session_name, agent_id),
            "-F",
            "#{pane_id}",
        ])
        .output()
        .await;

    match output {
        Ok(output) if output.status.success() && !output.stdout.is_empty() => AgentStatus::Running,
        _ => AgentStatus::Stopped,
    }
}

/// Gather task states from task queue
fn gather_task_states(colony_root: &Path) -> ColonyResult<Vec<TaskState>> {
    let task_queue = TaskQueue::new(colony_root);
    let tasks = task_queue.load_all_tasks()?;

    Ok(tasks
        .into_iter()
        .map(|task| TaskState {
            id: task.id.clone(),
            title: task.title.clone(),
            status: convert_task_status(&task.status),
            assigned_to: task.assigned_to.clone(),
            priority: format!("{:?}", task.priority).to_lowercase(),
            created_at: task.created_at.clone(),
        })
        .collect())
}

/// Convert internal task status to relay protocol status
fn convert_task_status(
    status: &crate::colony::tasks::TaskStatus,
) -> TaskStatus {
    match status {
        crate::colony::tasks::TaskStatus::Pending => TaskStatus::Pending,
        crate::colony::tasks::TaskStatus::Claimed => TaskStatus::Claimed,
        crate::colony::tasks::TaskStatus::InProgress => TaskStatus::InProgress,
        crate::colony::tasks::TaskStatus::Blocked => TaskStatus::Blocked,
        crate::colony::tasks::TaskStatus::Completed => TaskStatus::Completed,
        crate::colony::tasks::TaskStatus::Cancelled => TaskStatus::Cancelled,
    }
}

/// Gather recent messages from the message queue
fn gather_message_states(colony_root: &Path) -> ColonyResult<Vec<MessageState>> {
    let messages = messaging::load_all_messages(colony_root)?;

    // Take last 50 messages
    let recent_messages: Vec<MessageState> = messages
        .into_iter()
        .rev()
        .take(50)
        .map(|msg| MessageState {
            id: msg.id.clone(),
            from: msg.from.clone(),
            to: msg.to.clone(),
            content: msg.content.clone(),
            timestamp: msg.timestamp.clone(),
            message_type: format!("{:?}", msg.message_type).to_lowercase(),
        })
        .collect();

    Ok(recent_messages)
}
