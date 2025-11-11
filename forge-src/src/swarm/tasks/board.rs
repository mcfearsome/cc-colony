use colored::Colorize;
use std::collections::HashMap;

use super::queue::TaskStatistics;
use super::{Task, TaskPriority, TaskStatus};

/// Render a task board to the console
pub fn render_task_board(tasks: &[Task], _agent_assignments: &HashMap<String, Vec<Task>>) {
    if tasks.is_empty() {
        println!("No tasks defined");
        return;
    }

    // Header
    println!("\n{}", "═".repeat(100));
    println!("{}", "TASK BOARD".bold().cyan());
    println!("{}", "═".repeat(100));

    // Table header
    println!(
        "\n{:<12} {:<30} {:<15} {:<12} {:<10} {:<10}",
        "ID".bold(),
        "TITLE".bold(),
        "ASSIGNED TO".bold(),
        "STATUS".bold(),
        "PRIORITY".bold(),
        "PROGRESS".bold()
    );
    println!("{}", "─".repeat(100));

    // Task rows
    for task in tasks {
        let id = task.id.chars().take(10).collect::<String>();
        let title = task.title.chars().take(28).collect::<String>();
        let assigned = task
            .claimed_by
            .as_ref()
            .or(task.assigned_to.as_ref())
            .map(|s| s.chars().take(13).collect::<String>())
            .unwrap_or_else(|| "(unclaimed)".to_string());

        let status_text = format!("{} {}", task.status.emoji(), task.status.display());
        let status_colored = match task.status {
            TaskStatus::Pending => status_text.yellow(),
            TaskStatus::Claimed => status_text.blue(),
            TaskStatus::InProgress => status_text.cyan(),
            TaskStatus::Blocked => status_text.red(),
            TaskStatus::Completed => status_text.green(),
            TaskStatus::Cancelled => status_text.dimmed(),
        };

        let priority_colored = match task.priority {
            TaskPriority::Critical => "CRITICAL".red().bold(),
            TaskPriority::High => "HIGH".red(),
            TaskPriority::Medium => "MEDIUM".yellow(),
            TaskPriority::Low => "LOW".dimmed(),
        };

        let progress = if task.status == TaskStatus::Completed {
            "100%".green()
        } else if task.progress > 0 {
            format!("{}%", task.progress).cyan()
        } else {
            "-".dimmed()
        };

        println!(
            "{:<12} {:<30} {:<15} {:<22} {:<18} {:<10}",
            id, title, assigned, status_colored, priority_colored, progress
        );

        // Show blockers if any
        if !task.blockers.is_empty() {
            for blocker in &task.blockers {
                println!("             {} {}", "⚠".yellow(), blocker.yellow());
            }
        }

        // Show dependencies if pending
        if task.status == TaskStatus::Pending && !task.dependencies.is_empty() {
            let deps = task.dependencies.join(", ");
            println!(
                "             {} Depends on: {}",
                "↳".dimmed(),
                deps.dimmed()
            );
        }
    }

    println!("{}", "─".repeat(100));
}

/// Render task statistics
pub fn render_task_statistics(stats: &TaskStatistics) {
    println!("\n{}", "TASK STATISTICS".bold().cyan());
    println!("{}", "─".repeat(50));

    let completion = stats.completion_percentage();
    let progress_bar = render_progress_bar(completion, 30);

    println!("Total Tasks:        {}", stats.total.to_string().bold());
    println!(
        "Active Tasks:       {}",
        format!("{}", stats.active_count()).cyan().bold()
    );
    println!(
        "Completed:          {}",
        format!("{} ", stats.completed).green()
    );
    println!(
        "In Progress:        {}",
        format!("{}", stats.in_progress).cyan()
    );
    println!(
        "Claimed:            {}",
        format!("{}", stats.claimed).blue()
    );
    println!(
        "Pending:            {}",
        format!("{}", stats.pending).yellow()
    );
    println!("Blocked:            {}", format!("{}", stats.blocked).red());
    println!(
        "Cancelled:          {}",
        format!("{}", stats.cancelled).dimmed()
    );
    println!();
    println!("Overall Progress:   {} {:.1}%", progress_bar, completion);
    println!("{}", "─".repeat(50));
}

/// Render agent task assignments
pub fn render_agent_assignments(agent_assignments: &HashMap<String, Vec<Task>>) {
    if agent_assignments.is_empty() {
        return;
    }

    println!("\n{}", "AGENT ASSIGNMENTS".bold().cyan());
    println!("{}", "─".repeat(80));

    for (agent_id, tasks) in agent_assignments {
        let active_tasks: Vec<&Task> = tasks
            .iter()
            .filter(|t| {
                matches!(
                    t.status,
                    TaskStatus::Claimed | TaskStatus::InProgress | TaskStatus::Blocked
                )
            })
            .collect();

        if active_tasks.is_empty() {
            continue;
        }

        println!("\n{}", agent_id.bold());

        for task in active_tasks {
            let status_text = format!("{} {}", task.status.emoji(), task.status.display());
            let progress = if task.progress > 0 {
                format!("({}%)", task.progress).dimmed().to_string()
            } else {
                String::new()
            };

            println!(
                "  • {:<30} {} {}",
                task.title.chars().take(28).collect::<String>(),
                status_text,
                progress
            );
        }
    }

    println!("{}", "─".repeat(80));
}

/// Render a single task details
pub fn render_task_detail(task: &Task) {
    println!("\n{}", "═".repeat(80));
    println!("{} {}", "TASK:".bold().cyan(), task.title.bold());
    println!("{}", "═".repeat(80));

    println!("\n{:<20} {}", "ID:".bold(), task.id);
    println!("{:<20} {}", "Title:".bold(), task.title);
    println!("{:<20} {}", "Description:".bold(), task.description);
    println!();

    let status_text = format!("{} {}", task.status.emoji(), task.status.display());
    let status_colored = match task.status {
        TaskStatus::Pending => status_text.yellow(),
        TaskStatus::Claimed => status_text.blue(),
        TaskStatus::InProgress => status_text.cyan(),
        TaskStatus::Blocked => status_text.red(),
        TaskStatus::Completed => status_text.green(),
        TaskStatus::Cancelled => status_text.dimmed(),
    };

    println!("{:<20} {}", "Status:".bold(), status_colored);

    let priority_colored = match task.priority {
        TaskPriority::Critical => "CRITICAL".red().bold(),
        TaskPriority::High => "HIGH".red(),
        TaskPriority::Medium => "MEDIUM".yellow(),
        TaskPriority::Low => "LOW".dimmed(),
    };
    println!("{:<20} {}", "Priority:".bold(), priority_colored);

    if task.progress > 0 {
        let progress_bar = render_progress_bar(task.progress as f64, 30);
        println!(
            "{:<20} {} {}%",
            "Progress:".bold(),
            progress_bar,
            task.progress
        );
    }

    println!();

    if let Some(ref assigned) = task.assigned_to {
        println!("{:<20} {}", "Assigned To:".bold(), assigned);
    }

    if let Some(ref claimed) = task.claimed_by {
        println!("{:<20} {}", "Claimed By:".bold(), claimed);
    }

    if !task.dependencies.is_empty() {
        println!(
            "{:<20} {}",
            "Dependencies:".bold(),
            task.dependencies.join(", ")
        );
    }

    if !task.blockers.is_empty() {
        println!("{:<20}", "Blockers:".bold());
        for blocker in &task.blockers {
            println!("  {} {}", "⚠".yellow(), blocker);
        }
    }

    if !task.tags.is_empty() {
        println!("{:<20} {}", "Tags:".bold(), task.tags.join(", "));
    }

    println!();
    println!("{:<20} {}", "Created:".bold(), task.created_at);

    if let Some(ref claimed_at) = task.claimed_at {
        println!("{:<20} {}", "Claimed:".bold(), claimed_at);
    }

    if let Some(ref started_at) = task.started_at {
        println!("{:<20} {}", "Started:".bold(), started_at);
    }

    if let Some(ref completed_at) = task.completed_at {
        println!("{:<20} {}", "Completed:".bold(), completed_at);
    }

    println!("{:<20} {}", "Last Updated:".bold(), task.updated_at);

    println!("{}", "═".repeat(80));
}

/// Render a progress bar
fn render_progress_bar(percentage: f64, width: usize) -> String {
    let filled = ((percentage / 100.0) * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);

    let bar = format!("{}{}", "█".repeat(filled), "░".repeat(empty));

    if percentage >= 100.0 {
        bar.green().to_string()
    } else if percentage >= 75.0 {
        bar.cyan().to_string()
    } else if percentage >= 50.0 {
        bar.yellow().to_string()
    } else {
        bar.dimmed().to_string()
    }
}

/// Render a compact task list (for agent use)
pub fn render_compact_task_list(tasks: &[Task]) {
    if tasks.is_empty() {
        println!("No tasks available");
        return;
    }

    for (i, task) in tasks.iter().enumerate() {
        let priority_marker = match task.priority {
            TaskPriority::Critical => "!!!".red(),
            TaskPriority::High => "!!".red(),
            TaskPriority::Medium => "!".yellow(),
            TaskPriority::Low => " ".normal(),
        };

        println!(
            "{}. {} {} - {}",
            i + 1,
            priority_marker,
            task.id.bold(),
            task.title
        );

        if !task.description.is_empty() {
            let desc = task.description.chars().take(60).collect::<String>();
            println!("   {}", desc.dimmed());
        }
    }
}
