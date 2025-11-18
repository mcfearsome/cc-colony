//! CLI commands for shared state management

use crate::colony::state::{GitBackedState, SharedStateConfig, Task, TaskIdGenerator, TaskStatus, Workflow, WorkflowStatus, MemoryEntry, MemoryType};
use crate::error::{ColonyError, ColonyResult};
use crate::utils;
use chrono::Utc;
use colored::Colorize;

/// Initialize state backend from config
async fn init_state() -> ColonyResult<GitBackedState> {
    let config_path = std::path::Path::new("colony.yml");

    if !config_path.exists() {
        return Err(ColonyError::Colony(
            "colony.yml not found. Run 'colony init' first.".to_string(),
        ));
    }

    let config = crate::colony::ColonyConfig::load(config_path)?;

    let state_config = config.shared_state.ok_or_else(|| {
        ColonyError::Colony(
            "Shared state not configured. Add 'shared_state' section to colony.yml".to_string(),
        )
    })?;

    let repo_root = std::env::current_dir()?;
    GitBackedState::new(state_config, repo_root)
}

// ============================================================================
// Task Commands
// ============================================================================

/// List all tasks
pub async fn task_list() -> ColonyResult<()> {
    let state = init_state().await?;
    let tasks = state.get_tasks().await?;

    if tasks.is_empty() {
        println!("{}", "No tasks found".yellow());
        return Ok(());
    }

    println!("\n{}", "Tasks:".bold());
    println!("{}", "─".repeat(80));

    for task in tasks {
        let status_icon = match task.status {
            TaskStatus::Ready => "●".green(),
            TaskStatus::Blocked => "◆".red(),
            TaskStatus::InProgress => "◐".cyan(),
            TaskStatus::Completed => "✓".bright_green(),
            TaskStatus::Cancelled => "✗".bright_black(),
        };

        let assigned = task.assigned.as_deref().unwrap_or("unassigned");

        println!(
            "{} {} {} ({})",
            status_icon,
            task.id.bright_blue(),
            task.title.bold(),
            assigned.yellow()
        );

        if let Some(desc) = &task.description {
            println!("  {}", desc.dimmed());
        }

        if !task.blockers.is_empty() {
            println!("  {} {}", "Blocked by:".red(), task.blockers.join(", "));
        }
    }

    println!();
    Ok(())
}

/// List ready tasks (no blockers)
pub async fn task_ready() -> ColonyResult<()> {
    let state = init_state().await?;
    let tasks = state.get_ready_tasks().await?;

    if tasks.is_empty() {
        println!("{}", "No ready tasks found".yellow());
        return Ok(());
    }

    println!("\n{}", "Ready Tasks:".bold().green());
    println!("{}", "─".repeat(80));

    for task in tasks {
        let assigned = task.assigned.as_deref().unwrap_or("unassigned");

        println!(
            "● {} {} ({})",
            task.id.bright_blue(),
            task.title.bold(),
            assigned.yellow()
        );

        if let Some(desc) = &task.description {
            println!("  {}", desc.dimmed());
        }
    }

    println!();
    Ok(())
}

/// Show task details
pub async fn task_show(id: String) -> ColonyResult<()> {
    let state = init_state().await?;
    let task = state.get_task(&id).await?;

    match task {
        Some(t) => {
            println!("\n{}", "Task Details:".bold());
            println!("{}", "─".repeat(80));
            println!("{}: {}", "ID".bold(), t.id.bright_blue());
            println!("{}: {}", "Title".bold(), t.title);

            if let Some(desc) = &t.description {
                println!("{}: {}", "Description".bold(), desc);
            }

            let status_str = match t.status {
                TaskStatus::Ready => "Ready".green(),
                TaskStatus::Blocked => "Blocked".red(),
                TaskStatus::InProgress => "In Progress".cyan(),
                TaskStatus::Completed => "Completed".bright_green(),
                TaskStatus::Cancelled => "Cancelled".bright_black(),
            };
            println!("{}: {}", "Status".bold(), status_str);

            println!("{}: {}", "Created".bold(), t.created.format("%Y-%m-%d %H:%M:%S"));

            if let Some(assigned) = &t.assigned {
                println!("{}: {}", "Assigned".bold(), assigned.yellow());
            }

            if !t.blockers.is_empty() {
                println!("{}: {}", "Blocked by".bold(), t.blockers.join(", ").red());
            }

            if let Some(completed) = t.completed {
                println!("{}: {}", "Completed".bold(), completed.format("%Y-%m-%d %H:%M:%S"));
            }

            println!();
            Ok(())
        }
        None => {
            println!("{}", format!("Task '{}' not found", id).red());
            Err(ColonyError::Colony(format!("Task '{}' not found", id)))
        }
    }
}

/// Create a new task
pub async fn task_add(title: String, description: Option<String>, blockers: Vec<String>) -> ColonyResult<()> {
    let state = init_state().await?;

    let task = if blockers.is_empty() {
        let mut t = Task::new(title.clone());
        t.description = description;
        t
    } else {
        let mut t = Task::new_with_blockers(title.clone(), blockers.clone());
        t.description = description;
        t
    };

    let task_id = task.id.clone();
    state.add_task(task).await?;

    utils::success(&format!("Created task: {}", task_id.bright_blue()));
    println!("  Title: {}", title);
    if !blockers.is_empty() {
        println!("  Blockers: {}", blockers.join(", ").red());
    }

    Ok(())
}

/// Update task status
pub async fn task_update(id: String, status: String) -> ColonyResult<()> {
    let state = init_state().await?;

    let mut task = state.get_task(&id).await?
        .ok_or_else(|| ColonyError::Colony(format!("Task '{}' not found", id)))?;

    let new_status = match status.to_lowercase().as_str() {
        "ready" => TaskStatus::Ready,
        "blocked" => TaskStatus::Blocked,
        "in_progress" | "inprogress" | "in-progress" => TaskStatus::InProgress,
        "completed" | "complete" | "done" => {
            task.completed = Some(Utc::now());
            TaskStatus::Completed
        }
        "cancelled" | "cancel" => TaskStatus::Cancelled,
        _ => {
            return Err(ColonyError::Colony(format!(
                "Invalid status '{}'. Use: ready, blocked, in_progress, completed, cancelled",
                status
            )));
        }
    };

    let old_status = format!("{:?}", task.status);
    let new_status_str = format!("{:?}", new_status);
    task.status = new_status;
    state.update_task(task).await?;

    utils::success(&format!("Updated task {} status: {} → {}",
        id.bright_blue(),
        old_status.yellow(),
        new_status_str.green()
    ));

    Ok(())
}

/// Assign task to agent
pub async fn task_assign(id: String, agent_id: String) -> ColonyResult<()> {
    let state = init_state().await?;

    let mut task = state.get_task(&id).await?
        .ok_or_else(|| ColonyError::Colony(format!("Task '{}' not found", id)))?;

    task.assigned = Some(agent_id.clone());
    state.update_task(task).await?;

    utils::success(&format!("Assigned task {} to {}",
        id.bright_blue(),
        agent_id.yellow()
    ));

    Ok(())
}

/// Add blocker to task
pub async fn task_block(id: String, blocker: String) -> ColonyResult<()> {
    let state = init_state().await?;

    let mut task = state.get_task(&id).await?
        .ok_or_else(|| ColonyError::Colony(format!("Task '{}' not found", id)))?;

    if !task.blockers.contains(&blocker) {
        task.blockers.push(blocker.clone());
        task.status = TaskStatus::Blocked;
        state.update_task(task).await?;

        utils::success(&format!("Added blocker {} to task {}",
            blocker.red(),
            id.bright_blue()
        ));
    } else {
        println!("{}", format!("Task {} already blocked by {}", id, blocker).yellow());
    }

    Ok(())
}

// ============================================================================
// Workflow Commands
// ============================================================================

/// List all workflows
pub async fn workflow_list() -> ColonyResult<()> {
    let state = init_state().await?;
    let workflows = state.get_workflows().await?;

    if workflows.is_empty() {
        println!("{}", "No workflows found".yellow());
        return Ok(());
    }

    println!("\n{}", "Workflows:".bold());
    println!("{}", "─".repeat(80));

    for workflow in workflows {
        let status_icon = match workflow.status {
            WorkflowStatus::Pending => "○".yellow(),
            WorkflowStatus::Running => "◐".cyan(),
            WorkflowStatus::Completed => "✓".bright_green(),
            WorkflowStatus::Failed => "✗".red(),
            WorkflowStatus::Cancelled => "✗".bright_black(),
        };

        println!(
            "{} {} {}",
            status_icon,
            workflow.id.bright_blue(),
            workflow.name.bold()
        );

        if let Some(step) = &workflow.current_step {
            println!("  Current step: {}", step.cyan());
        }
    }

    println!();
    Ok(())
}

/// Show workflow details
pub async fn workflow_show(id: String) -> ColonyResult<()> {
    let state = init_state().await?;
    let workflow = state.get_workflow(&id).await?;

    match workflow {
        Some(w) => {
            println!("\n{}", "Workflow Details:".bold());
            println!("{}", "─".repeat(80));
            println!("{}: {}", "ID".bold(), w.id.bright_blue());
            println!("{}: {}", "Name".bold(), w.name);

            let status_str = match w.status {
                WorkflowStatus::Pending => "Pending".yellow(),
                WorkflowStatus::Running => "Running".cyan(),
                WorkflowStatus::Completed => "Completed".bright_green(),
                WorkflowStatus::Failed => "Failed".red(),
                WorkflowStatus::Cancelled => "Cancelled".bright_black(),
            };
            println!("{}: {}", "Status".bold(), status_str);

            println!("{}: {}", "Started".bold(), w.started.format("%Y-%m-%d %H:%M:%S"));

            if let Some(completed) = w.completed {
                println!("{}: {}", "Completed".bold(), completed.format("%Y-%m-%d %H:%M:%S"));
            }

            if let Some(step) = &w.current_step {
                println!("{}: {}", "Current Step".bold(), step.cyan());
            }

            if !w.steps.is_empty() {
                println!("\n{}", "Steps:".bold());
                for (name, step) in &w.steps {
                    let status_icon = match step.status {
                        crate::colony::state::StepStatus::Pending => "○".yellow(),
                        crate::colony::state::StepStatus::Running => "◐".cyan(),
                        crate::colony::state::StepStatus::Completed => "✓".bright_green(),
                        crate::colony::state::StepStatus::Failed => "✗".red(),
                        crate::colony::state::StepStatus::Skipped => "⊘".dimmed(),
                    };
                    println!("  {} {}", status_icon, name);
                }
            }

            println!();
            Ok(())
        }
        None => {
            println!("{}", format!("Workflow '{}' not found", id).red());
            Err(ColonyError::Colony(format!("Workflow '{}' not found", id)))
        }
    }
}

/// Create a new workflow
pub async fn workflow_add(name: String) -> ColonyResult<()> {
    let state = init_state().await?;

    let workflow = Workflow::new(name.clone());
    let workflow_id = workflow.id.clone();

    state.add_workflow(workflow).await?;

    utils::success(&format!("Created workflow: {}", workflow_id.bright_blue()));
    println!("  Name: {}", name);

    Ok(())
}

/// Update workflow status
pub async fn workflow_update(id: String, status: String) -> ColonyResult<()> {
    let state = init_state().await?;

    let mut workflow = state.get_workflow(&id).await?
        .ok_or_else(|| ColonyError::Colony(format!("Workflow '{}' not found", id)))?;

    let new_status = match status.to_lowercase().as_str() {
        "pending" => WorkflowStatus::Pending,
        "running" | "run" => WorkflowStatus::Running,
        "completed" | "complete" | "done" => {
            workflow.completed = Some(Utc::now());
            WorkflowStatus::Completed
        }
        "failed" | "fail" | "error" => {
            workflow.completed = Some(Utc::now());
            WorkflowStatus::Failed
        }
        _ => {
            return Err(ColonyError::Colony(format!(
                "Invalid status '{}'. Use: pending, running, completed, failed",
                status
            )));
        }
    };

    let old_status = format!("{:?}", workflow.status);
    let new_status_str = format!("{:?}", new_status);
    workflow.status = new_status;
    state.update_workflow(workflow).await?;

    utils::success(&format!("Updated workflow {} status: {} → {}",
        id.bright_blue(),
        old_status.yellow(),
        new_status_str.green()
    ));

    Ok(())
}

// ============================================================================
// Memory Commands
// ============================================================================

/// Add memory entry
pub async fn memory_add(entry_type: String, content: String, key: Option<String>, value: Option<String>) -> ColonyResult<()> {
    let state = init_state().await?;

    let mem_type = match entry_type.to_lowercase().as_str() {
        "context" | "ctx" => MemoryType::Context,
        "learned" | "learn" => MemoryType::Learned,
        "todo" | "task" => MemoryType::Todo,
        "note" | "notes" => MemoryType::Note,
        _ => {
            return Err(ColonyError::Colony(format!(
                "Invalid memory type '{}'. Use: context, learned, todo, note",
                entry_type
            )));
        }
    };

    let entry = MemoryEntry {
        timestamp: Utc::now(),
        entry_type: mem_type,
        key,
        value,
        content: Some(content.clone()),
    };

    state.add_memory(entry).await?;

    utils::success(&format!("Added {} memory entry", entry_type.cyan()));
    println!("  Content: {}", content);

    Ok(())
}

/// Search memory entries
pub async fn memory_search(_query: String) -> ColonyResult<()> {
    let state = init_state().await?;
    let entries = state.get_memory().await?;

    if entries.is_empty() {
        println!("{}", "No memory entries found".yellow());
        return Ok(());
    }

    println!("\n{}", "Memory Entries:".bold());
    println!("{}", "─".repeat(80));

    for entry in entries {
        let type_str = match entry.entry_type {
            MemoryType::Context => "CONTEXT".bright_blue(),
            MemoryType::Learned => "LEARNED".green(),
            MemoryType::Todo => "TODO".yellow(),
            MemoryType::Note => "NOTE".cyan(),
        };

        println!(
            "[{}] {}",
            type_str,
            entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string().dimmed()
        );

        if let Some(key) = &entry.key {
            println!("  Key: {}", key.bold());
        }
        if let Some(value) = &entry.value {
            println!("  Value: {}", value);
        }
        if let Some(content) = &entry.content {
            println!("  {}", content);
        }
        println!();
    }

    Ok(())
}

// ============================================================================
// Sync Commands
// ============================================================================

/// Pull latest state from remote
pub async fn pull() -> ColonyResult<()> {
    let state = init_state().await?;

    utils::info("Pulling latest state from remote...");
    state.pull().await?;
    utils::success("State synced from remote");

    Ok(())
}

/// Push local state to remote
pub async fn push() -> ColonyResult<()> {
    utils::info("Pushing local state to remote...");

    // For push, we'd need to expose git operations from GitBackedState
    // For now, we can use git directly
    let state_dir = std::env::current_dir()?.join(".colony/state");

    if !state_dir.exists() {
        return Err(ColonyError::Colony("State directory not found".to_string()));
    }

    let output = std::process::Command::new("git")
        .args(["push"])
        .current_dir(&state_dir)
        .output()
        .map_err(|e| ColonyError::Colony(format!("Failed to push state: {}", e)))?;

    if output.status.success() {
        utils::success("State pushed to remote");
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(ColonyError::Colony(format!("Failed to push state: {}", error)))
    }
}

/// Full sync (pull + push)
pub async fn sync() -> ColonyResult<()> {
    pull().await?;
    push().await?;
    utils::success("State fully synced");
    Ok(())
}
