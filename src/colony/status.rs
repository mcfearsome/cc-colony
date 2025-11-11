use colored::Colorize;
use std::path::Path;

use crate::colony::{AgentStatus, ColonyConfig, ColonyController};
use crate::error::ColonyResult;
use crate::utils;

/// Show status of all agents
pub async fn run() -> ColonyResult<()> {
    let config_path = Path::new("colony.yml");

    if !config_path.exists() {
        return Err(crate::error::ColonyError::Colony(
            "colony.yml not found. Run 'forge colony init' first.".to_string(),
        ));
    }

    // Load configuration
    let config = ColonyConfig::load(config_path)?;

    // Create controller
    let mut controller = ColonyController::new(config)?;
    controller.initialize_agents()?;

    // Load state
    controller.load_state()?;

    utils::header("Forge Colony Status");

    if controller.agents().is_empty() {
        utils::info("No agents configured");
        return Ok(());
    }

    // Print header
    println!(
        "{:<15} {:<20} {:<12} {:<10}",
        "AGENT ID", "ROLE", "STATUS", "PID"
    );
    println!("{}", "─".repeat(70));

    // Print each agent
    for agent in controller.agents().values() {
        let status_str = format_status(&agent.status);
        let pid_str = agent
            .pid
            .map(|p| p.to_string())
            .unwrap_or_else(|| "-".to_string());

        println!(
            "{:<15} {:<20} {:<12} {:<10}",
            agent.id(),
            truncate(&agent.config.role, 20),
            status_str,
            pid_str
        );
    }

    println!();

    // Summary
    let total = controller.agents().len();
    let running = controller
        .agents()
        .values()
        .filter(|a| a.status == AgentStatus::Running)
        .count();
    let completed = controller
        .agents()
        .values()
        .filter(|a| a.status == AgentStatus::Completed)
        .count();
    let failed = controller
        .agents()
        .values()
        .filter(|a| a.status == AgentStatus::Failed)
        .count();

    println!(
        "Total: {} | Running: {} | Completed: {} | Failed: {}",
        total, running, completed, failed
    );

    Ok(())
}

fn format_status(status: &AgentStatus) -> String {
    match status {
        AgentStatus::Idle => "idle".white().to_string(),
        AgentStatus::Running => "running".green().to_string(),
        AgentStatus::Completed => "completed".blue().to_string(),
        AgentStatus::Failed => "failed".red().to_string(),
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
