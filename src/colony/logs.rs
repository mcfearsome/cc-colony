use std::path::Path;

use crate::colony::{ColonyConfig, ColonyController};
use crate::error::ColonyResult;
use crate::utils;

/// View logs for an agent
pub async fn run(agent_id: Option<String>) -> ColonyResult<()> {
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

    match agent_id {
        Some(id) => {
            // Show logs for specific agent
            show_agent_logs(&controller, &id)?;
        }
        None => {
            // List all available logs
            list_agent_logs(&controller)?;
        }
    }

    Ok(())
}

fn show_agent_logs(controller: &ColonyController, agent_id: &str) -> ColonyResult<()> {
    let agent = controller
        .get_agent(agent_id)
        .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

    utils::header(&format!("Logs for agent: {}", agent_id));

    if !agent.log_path.exists() {
        utils::info("No logs available yet");
        utils::info(&format!("Log file: {}", agent.log_path.display()));
        return Ok(());
    }

    // Read and display log file
    let contents = std::fs::read_to_string(&agent.log_path)?;

    if contents.is_empty() {
        utils::info("Log file is empty");
    } else {
        println!("{}", contents);
    }

    println!();
    utils::info(&format!("Log file: {}", agent.log_path.display()));

    Ok(())
}

fn list_agent_logs(controller: &ColonyController) -> ColonyResult<()> {
    utils::header("Available agent logs");

    if controller.agents().is_empty() {
        utils::info("No agents configured");
        return Ok(());
    }

    println!("{:<15} {:<10} PATH", "AGENT ID", "SIZE");
    println!("{}", "─".repeat(70));

    for agent in controller.agents().values() {
        let size = if agent.log_path.exists() {
            let metadata = std::fs::metadata(&agent.log_path)?;
            format_size(metadata.len())
        } else {
            "-".to_string()
        };

        println!(
            "{:<15} {:<10} {}",
            agent.id(),
            size,
            agent.log_path.display()
        );
    }

    println!();
    utils::info("Use 'forge colony logs <agent-id>' to view specific logs");

    Ok(())
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;

    if bytes < KB {
        format!("{} B", bytes)
    } else if bytes < MB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    }
}
