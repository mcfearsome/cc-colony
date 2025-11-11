use std::path::Path;

use crate::colony::config::ColonyConfig;
use crate::colony::controller::ColonyController;
use crate::colony::tasks::board::{
    render_agent_assignments, render_compact_task_list, render_task_board, render_task_detail,
    render_task_statistics,
};
use crate::colony::tasks::queue::TaskQueue;
use crate::colony::tasks::{Task, TaskPriority, TaskStatus};
use crate::error::ColonyResult;
use crate::utils;

/// Helper function to load task queue (reduces boilerplate)
fn load_task_queue() -> ColonyResult<TaskQueue> {
    let config_path = Path::new("colony.yml");

    if !config_path.exists() {
        return Err(crate::error::ColonyError::Colony(
            "colony.yml not found. Run 'forge colony init' first.".to_string(),
        ));
    }

    let config = ColonyConfig::load(config_path)?;
    let controller = ColonyController::new(config)?;
    Ok(TaskQueue::new(controller.colony_root()))
}

/// List all tasks
pub async fn list_tasks(status_filter: Option<String>, compact: bool) -> ColonyResult<()> {
    let queue = load_task_queue()?;

    // Load tasks based on filter
    let tasks = if let Some(status_str) = status_filter {
        let status = parse_task_status(&status_str)?;
        queue.load_tasks_by_status(&status)?
    } else {
        queue.load_all_tasks()?
    };

    if compact {
        // Render compact view
        render_compact_task_list(&tasks);
    } else {
        // Get agent assignments
        let assignments = queue.get_agent_assignments()?;

        // Render task board
        render_task_board(&tasks, &assignments);

        // Show statistics
        let stats = queue.get_statistics()?;
        render_task_statistics(&stats);

        // Show agent assignments
        render_agent_assignments(&assignments);
    }

    Ok(())
}

/// Show details for a specific task
pub async fn show_task(task_id: String) -> ColonyResult<()> {
    let queue = load_task_queue()?;

    match queue.load_task(&task_id)? {
        Some(task) => {
            render_task_detail(&task);
            Ok(())
        }
        None => Err(crate::error::ColonyError::Colony(format!(
            "Task '{}' not found",
            task_id
        ))),
    }
}

/// Claim a task for an agent
pub async fn claim_task(task_id: String, agent_id: String) -> ColonyResult<()> {
    let queue = load_task_queue()?;

    // Load the task
    let mut task = match queue.load_task(&task_id)? {
        Some(t) => t,
        None => {
            return Err(crate::error::ColonyError::Colony(format!(
                "Task '{}' not found",
                task_id
            )))
        }
    };

    // Check if it can be claimed
    let completed_ids = queue.get_completed_task_ids()?;
    if !task.can_claim(&agent_id, &completed_ids) {
        let reason = if task.status != TaskStatus::Pending {
            format!("Task status is {}, not pending", task.status.display())
        } else if !task.dependencies.is_empty() {
            format!(
                "Task has uncompleted dependencies: {}",
                task.dependencies.join(", ")
            )
        } else if let Some(ref assigned) = task.assigned_to {
            if assigned != "auto" && assigned != &agent_id {
                format!("Task is assigned to {}", assigned)
            } else {
                "Unknown reason".to_string()
            }
        } else {
            "Unknown reason".to_string()
        };

        return Err(crate::error::ColonyError::Colony(format!(
            "Cannot claim task '{}': {}",
            task_id, reason
        )));
    }

    // Claim it
    task.claim(&agent_id);
    queue.update_task(&task)?;

    utils::success(&format!(
        "Task '{}' claimed by agent '{}'",
        task_id, agent_id
    ));

    Ok(())
}

/// Update task progress
pub async fn update_task_progress(task_id: String, progress: u8) -> ColonyResult<()> {
    let queue = load_task_queue()?;

    let mut task = match queue.load_task(&task_id)? {
        Some(t) => t,
        None => {
            return Err(crate::error::ColonyError::Colony(format!(
                "Task '{}' not found",
                task_id
            )))
        }
    };

    task.update_progress(progress);
    queue.update_task(&task)?;

    utils::success(&format!(
        "Task '{}' progress updated to {}%",
        task_id, progress
    ));

    Ok(())
}

/// Mark task as blocked
pub async fn block_task(task_id: String, reason: String) -> ColonyResult<()> {
    let queue = load_task_queue()?;

    let mut task = match queue.load_task(&task_id)? {
        Some(t) => t,
        None => {
            return Err(crate::error::ColonyError::Colony(format!(
                "Task '{}' not found",
                task_id
            )))
        }
    };

    task.block(reason.clone());
    queue.update_task(&task)?;

    utils::warning(&format!("Task '{}' marked as blocked: {}", task_id, reason));

    Ok(())
}

/// Complete a task
pub async fn complete_task(task_id: String) -> ColonyResult<()> {
    let queue = load_task_queue()?;

    let mut task = match queue.load_task(&task_id)? {
        Some(t) => t,
        None => {
            return Err(crate::error::ColonyError::Colony(format!(
                "Task '{}' not found",
                task_id
            )))
        }
    };

    task.complete();
    queue.update_task(&task)?;

    utils::success(&format!("Task '{}' marked as completed!", task_id));

    Ok(())
}

/// Create a new task
pub async fn create_task(
    task_id: String,
    title: String,
    description: String,
    assigned_to: Option<String>,
    priority: Option<String>,
) -> ColonyResult<()> {
    let queue = load_task_queue()?;

    // Check if task already exists
    if queue.load_task(&task_id)?.is_some() {
        return Err(crate::error::ColonyError::Colony(format!(
            "Task '{}' already exists",
            task_id
        )));
    }

    let mut task = Task::new(task_id.clone(), title, description);

    if let Some(agent) = assigned_to {
        task.assigned_to = Some(agent);
    }

    if let Some(priority_str) = priority {
        task.priority = parse_task_priority(&priority_str)?;
    }

    queue.create_task(task)?;

    utils::success(&format!("Task '{}' created successfully!", task_id));

    Ok(())
}

/// Parse task status from string
fn parse_task_status(status: &str) -> ColonyResult<TaskStatus> {
    match status.to_lowercase().as_str() {
        "pending" => Ok(TaskStatus::Pending),
        "claimed" => Ok(TaskStatus::Claimed),
        "in_progress" | "inprogress" | "working" => Ok(TaskStatus::InProgress),
        "blocked" => Ok(TaskStatus::Blocked),
        "completed" | "done" => Ok(TaskStatus::Completed),
        "cancelled" => Ok(TaskStatus::Cancelled),
        _ => Err(crate::error::ColonyError::Colony(format!(
            "Invalid task status: {}. Must be one of: pending, claimed, in_progress, blocked, completed, cancelled",
            status
        ))),
    }
}

/// Parse task priority from string
fn parse_task_priority(priority: &str) -> ColonyResult<TaskPriority> {
    match priority.to_lowercase().as_str() {
        "low" => Ok(TaskPriority::Low),
        "medium" | "med" => Ok(TaskPriority::Medium),
        "high" => Ok(TaskPriority::High),
        "critical" | "crit" => Ok(TaskPriority::Critical),
        _ => Err(crate::error::ColonyError::Colony(format!(
            "Invalid task priority: {}. Must be one of: low, medium, high, critical",
            priority
        ))),
    }
}

/// Unblock a task
pub async fn unblock_task(task_id: String) -> ColonyResult<()> {
    let queue = load_task_queue()?;

    let mut task = match queue.load_task(&task_id)? {
        Some(t) => t,
        None => {
            return Err(crate::error::ColonyError::Colony(format!(
                "Task '{}' not found",
                task_id
            )))
        }
    };

    if task.status != TaskStatus::Blocked {
        return Err(crate::error::ColonyError::Colony(format!(
            "Task '{}' is not blocked (status: {})",
            task_id,
            task.status.display()
        )));
    }

    task.unblock();
    queue.update_task(&task)?;

    utils::success(&format!("Task '{}' unblocked and resumed", task_id));

    Ok(())
}

/// Cancel a task
pub async fn cancel_task(task_id: String) -> ColonyResult<()> {
    let queue = load_task_queue()?;

    let mut task = match queue.load_task(&task_id)? {
        Some(t) => t,
        None => {
            return Err(crate::error::ColonyError::Colony(format!(
                "Task '{}' not found",
                task_id
            )))
        }
    };

    if task.status == TaskStatus::Completed {
        return Err(crate::error::ColonyError::Colony(format!(
            "Cannot cancel completed task '{}'",
            task_id
        )));
    }

    if task.status == TaskStatus::Cancelled {
        return Err(crate::error::ColonyError::Colony(format!(
            "Task '{}' is already cancelled",
            task_id
        )));
    }

    task.cancel();
    queue.update_task(&task)?;

    utils::warning(&format!("Task '{}' has been cancelled", task_id));

    Ok(())
}

/// Delete a task
pub async fn delete_task(task_id: String) -> ColonyResult<()> {
    let queue = load_task_queue()?;

    // Check if task exists
    if queue.load_task(&task_id)?.is_none() {
        return Err(crate::error::ColonyError::Colony(format!(
            "Task '{}' not found",
            task_id
        )));
    }

    // Confirm deletion
    if !utils::confirm(&format!(
        "Are you sure you want to delete task '{}'?",
        task_id
    )) {
        utils::info("Deletion cancelled");
        return Ok(());
    }

    queue.delete_task(&task_id)?;

    utils::success(&format!("Task '{}' deleted successfully", task_id));

    Ok(())
}

/// List tasks for a specific agent
pub async fn list_tasks_for_agent(agent_id: String) -> ColonyResult<()> {
    let queue = load_task_queue()?;

    let tasks = queue.load_tasks_for_agent(&agent_id)?;

    if tasks.is_empty() {
        println!("No tasks found for agent '{}'", agent_id);
        return Ok(());
    }

    utils::header(&format!("Tasks for Agent: {}", agent_id));
    println!();

    render_compact_task_list(&tasks);

    utils::success(&format!(
        "Found {} task(s) for agent '{}'",
        tasks.len(),
        agent_id
    ));

    Ok(())
}

/// List tasks an agent can claim
pub async fn list_claimable_tasks(agent_id: String) -> ColonyResult<()> {
    let queue = load_task_queue()?;

    let tasks = queue.find_claimable_tasks(&agent_id)?;

    if tasks.is_empty() {
        println!("No claimable tasks available for agent '{}'", agent_id);
        return Ok(());
    }

    utils::header(&format!("Claimable Tasks for Agent: {}", agent_id));
    println!();

    render_compact_task_list(&tasks);

    utils::success(&format!(
        "Found {} claimable task(s) for agent '{}'",
        tasks.len(),
        agent_id
    ));

    Ok(())
}
