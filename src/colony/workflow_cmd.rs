use crate::colony::state::TaskIdGenerator;
use crate::colony::workflow::{
    topological_sort, WorkflowDefinition, WorkflowRun, WorkflowRunStatus, WorkflowStorage,
};
use crate::error::ColonyResult;
use crate::utils;
use chrono::Utc;
use colored::Colorize;
use std::path::Path;

/// List all workflows
pub fn list_workflows() -> ColonyResult<()> {
    let colony_root = Path::new(".colony");
    let storage = WorkflowStorage::new(colony_root);

    let workflows = storage.list_workflows()?;

    if workflows.is_empty() {
        utils::info("No workflows defined");
        println!("\nTo create a workflow, add a YAML file to .colony/workflows/");
        println!("See documentation for workflow definition format.");
        return Ok(());
    }

    utils::header("Workflows");
    println!();

    for workflow in workflows {
        let trigger_info = match &workflow.trigger {
            Some(crate::colony::workflow::WorkflowTrigger::Manual) => "manual".to_string(),
            Some(crate::colony::workflow::WorkflowTrigger::Schedule { cron }) => {
                format!("schedule: {}", cron)
            }
            Some(crate::colony::workflow::WorkflowTrigger::Webhook { path }) => {
                format!("webhook: {}", path)
            }
            None => "manual".to_string(),
        };

        println!("  {} {}", workflow.name.bold(), format!("({})", trigger_info).dimmed());

        if let Some(desc) = &workflow.description {
            println!("    {}", desc.dimmed());
        }

        println!("    Steps: {}", workflow.steps.len());
        println!();
    }

    Ok(())
}

/// Show workflow details
pub fn show_workflow(name: &str) -> ColonyResult<()> {
    let colony_root = Path::new(".colony");
    let storage = WorkflowStorage::new(colony_root);

    let workflow = storage.load_workflow(name)?;

    utils::header(&format!("Workflow: {}", workflow.name));
    println!();

    if let Some(desc) = &workflow.description {
        println!("Description: {}", desc);
        println!();
    }

    // Show trigger
    print!("Trigger: ");
    match &workflow.trigger {
        Some(crate::colony::workflow::WorkflowTrigger::Manual) => println!("Manual"),
        Some(crate::colony::workflow::WorkflowTrigger::Schedule { cron }) => {
            println!("Schedule ({})", cron)
        }
        Some(crate::colony::workflow::WorkflowTrigger::Webhook { path }) => {
            println!("Webhook ({})", path)
        }
        None => println!("Manual"),
    }
    println!();

    // Show input schema if defined
    if let Some(input) = &workflow.input {
        println!("Input Schema:");
        println!("{}", serde_json::to_string_pretty(&input.schema).unwrap_or_default());
        println!();
    }

    // Show steps with dependency information
    println!("Steps:");
    println!();

    // Get topological ordering
    let levels = topological_sort(&workflow)?;

    for (level_idx, level) in levels.iter().enumerate() {
        println!("  Level {}:", level_idx);
        for step_name in level {
            if let Some(step) = workflow.steps.iter().find(|s| &s.name == step_name) {
                println!("    {} {}", "●".green(), step.name.bold());
                println!("      Agent: {}", step.agent);

                if let Some(deps) = &step.depends_on {
                    println!("      Depends on: {}", deps.join(", "));
                }

                if let Some(parallel) = step.parallel {
                    println!("      Parallel: {} instances", parallel);
                }

                if let Some(timeout) = &step.timeout {
                    println!("      Timeout: {}", timeout);
                }

                if let Some(retry) = &step.retry {
                    println!("      Retry: {} attempts", retry.max_attempts);
                }

                println!();
            }
        }
    }

    // Show error handlers if defined
    if let Some(handlers) = &workflow.error_handling {
        println!("Error Handlers:");
        for handler in handlers {
            println!("  {}: {} ({})", handler.step, handler.instructions, handler.agent);
        }
        println!();
    }

    Ok(())
}

/// Run a workflow
pub fn run_workflow(name: &str, input_json: Option<&str>) -> ColonyResult<()> {
    let colony_root = Path::new(".colony");
    let storage = WorkflowStorage::new(colony_root);

    // Load workflow definition
    let workflow = storage.load_workflow(name)?;

    // Parse input if provided
    let input = if let Some(input_str) = input_json {
        serde_json::from_str(input_str).map_err(|e| {
            crate::error::ColonyError::Colony(format!("Invalid input JSON: {}", e))
        })?
    } else {
        serde_json::Value::Null
    };

    // Create workflow run
    let run_id = TaskIdGenerator::generate("run");
    let run = WorkflowRun {
        id: run_id.clone(),
        workflow_name: workflow.name.clone(),
        status: WorkflowRunStatus::Pending,
        input: Some(input),
        started_at: Utc::now(),
        completed_at: None,
        steps: vec![],
        error: None,
    };

    // Save the run
    storage.save_run(&run)?;

    utils::success(&format!("Workflow run created: {}", run_id));
    println!();
    println!("Run ID: {}", run_id.bold());
    println!("Workflow: {}", workflow.name);
    println!("Status: {}", "pending".yellow());
    println!();
    println!("Note: Workflow execution engine coming soon!");
    println!("For now, you can manually execute steps based on the workflow definition.");
    println!();
    println!("Use 'colony workflow status {}' to check progress", run_id);

    Ok(())
}

/// Show workflow run status
pub fn show_run_status(run_id: &str) -> ColonyResult<()> {
    let colony_root = Path::new(".colony");
    let storage = WorkflowStorage::new(colony_root);

    let run = storage.load_run(run_id)?;

    utils::header(&format!("Workflow Run: {}", run.id));
    println!();

    println!("Workflow: {}", run.workflow_name);
    println!("Status: {}", format_status(&run.status));
    println!("Started: {}", run.started_at.format("%Y-%m-%d %H:%M:%S UTC"));

    if let Some(completed) = run.completed_at {
        println!("Completed: {}", completed.format("%Y-%m-%d %H:%M:%S UTC"));
        let duration = completed.signed_duration_since(run.started_at);
        println!("Duration: {}", format_duration(duration.num_seconds()));
    }

    println!();

    if let Some(input) = &run.input {
        if !input.is_null() {
            println!("Input:");
            println!("{}", serde_json::to_string_pretty(input).unwrap_or_default());
            println!();
        }
    }

    if !run.steps.is_empty() {
        println!("Steps:");
        println!();

        for step in &run.steps {
            let status_icon = match step.status {
                crate::colony::workflow::StepStatus::Pending => "○".dimmed().to_string(),
                crate::colony::workflow::StepStatus::Running => "◐".cyan().to_string(),
                crate::colony::workflow::StepStatus::Completed => "✓".green().to_string(),
                crate::colony::workflow::StepStatus::Failed => "✗".red().to_string(),
                crate::colony::workflow::StepStatus::Skipped => "⊘".dimmed().to_string(),
                crate::colony::workflow::StepStatus::Retrying => "↻".yellow().to_string(),
            };

            println!("  {} {} ({})", status_icon, step.step_name.bold(), step.agent);

            if let Some(started) = step.started_at {
                println!("    Started: {}", started.format("%H:%M:%S"));
            }

            if let Some(completed) = step.completed_at {
                println!("    Completed: {}", completed.format("%H:%M:%S"));
            }

            if step.attempt > 1 {
                println!("    Attempt: {}", step.attempt);
            }

            if let Some(error) = &step.error {
                println!("    {}: {}", "Error".red(), error);
            }

            println!();
        }
    }

    if let Some(error) = &run.error {
        println!("{}: {}", "Error".red(), error);
        println!();
    }

    Ok(())
}

/// List workflow run history
pub fn list_run_history(workflow_name: &str, limit: Option<usize>) -> ColonyResult<()> {
    let colony_root = Path::new(".colony");
    let storage = WorkflowStorage::new(colony_root);

    let mut runs = storage.list_runs(workflow_name)?;

    if runs.is_empty() {
        utils::info(&format!("No runs found for workflow '{}'", workflow_name));
        return Ok(());
    }

    // Apply limit if specified
    let total_runs = runs.len();
    if let Some(n) = limit {
        runs.truncate(n);
    }

    utils::header(&format!("Run History: {}", workflow_name));
    println!();

    for run in &runs {
        let status_str = format_status(&run.status);
        let duration_str = if let Some(completed) = run.completed_at {
            let duration = completed.signed_duration_since(run.started_at);
            format!(" ({})", format_duration(duration.num_seconds()))
        } else {
            String::new()
        };

        println!(
            "  {} {} - {}{}",
            run.id[..8].dimmed(),
            run.started_at.format("%Y-%m-%d %H:%M:%S"),
            status_str,
            duration_str
        );
    }

    println!();
    println!("Showing {} of {} total runs", runs.len(), total_runs);

    Ok(())
}

/// Cancel a workflow run
pub fn cancel_run(run_id: &str) -> ColonyResult<()> {
    let colony_root = Path::new(".colony");
    let storage = WorkflowStorage::new(colony_root);

    let mut run = storage.load_run(run_id)?;

    if !matches!(run.status, WorkflowRunStatus::Pending | WorkflowRunStatus::Running) {
        return Err(crate::error::ColonyError::Colony(format!(
            "Cannot cancel workflow run with status '{}'",
            run.status
        )));
    }

    run.status = WorkflowRunStatus::Cancelled;
    run.completed_at = Some(Utc::now());

    storage.save_run(&run)?;

    utils::success(&format!("Cancelled workflow run: {}", run_id));

    Ok(())
}

/// Format workflow run status with color
fn format_status(status: &WorkflowRunStatus) -> String {
    match status {
        WorkflowRunStatus::Pending => "pending".yellow().to_string(),
        WorkflowRunStatus::Running => "running".cyan().to_string(),
        WorkflowRunStatus::Completed => "completed".green().to_string(),
        WorkflowRunStatus::Failed => "failed".red().to_string(),
        WorkflowRunStatus::Cancelled => "cancelled".dimmed().to_string(),
    }
}

/// Format duration in seconds to human-readable string
fn format_duration(seconds: i64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    }
}
