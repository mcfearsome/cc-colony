use colored::Colorize;
use std::env;

use crate::error::{ForgeError, ForgeResult};
use crate::utils;

use super::messaging::{load_all_messages, load_messages_for_agent, MessageType};

/// Helper to format the sender badge
fn format_from_badge(from: &str) -> colored::ColoredString {
    if from == "system" {
        from.yellow().bold()
    } else {
        from.cyan()
    }
}

/// Format message type as a colored badge
fn format_message_type(message_type: &MessageType) -> String {
    match message_type {
        MessageType::Info => "[INFO]".blue().to_string(),
        MessageType::Task => "[TASK]".cyan().bold().to_string(),
        MessageType::Question => "[QUESTION]".magenta().to_string(),
        MessageType::Answer => "[ANSWER]".green().to_string(),
        MessageType::Completed => "[COMPLETED]".green().bold().to_string(),
        MessageType::Error => "[ERROR]".red().bold().to_string(),
    }
}

/// List messages for a specific agent
pub async fn list_messages(agent_id: String) -> ForgeResult<()> {
    let current_dir = env::current_dir()?;
    let swarm_root = current_dir.join(".forge-swarm");

    if !swarm_root.exists() {
        return Err(ForgeError::Swarm(
            "No swarm found. Run 'forge swarm init' first.".to_string(),
        ));
    }

    let messages = load_messages_for_agent(&swarm_root, &agent_id)?;

    if messages.is_empty() {
        println!("No messages for agent '{}'", agent_id.cyan());
        return Ok(());
    }

    utils::header(&format!("Messages for Agent: {}", agent_id));
    println!();

    for message in &messages {
        let type_badge = format_message_type(&message.message_type);
        let from_badge = format_from_badge(&message.from);

        let to_badge = if message.to == "all" {
            "[BROADCAST]".yellow().to_string()
        } else if message.to == agent_id {
            "[DIRECT]".green().to_string()
        } else {
            format!("[TO: {}]", message.to).dimmed().to_string()
        };

        println!(
            "{} {} {} {}",
            type_badge,
            from_badge,
            to_badge,
            message.timestamp.dimmed()
        );
        println!("  {}", message.content);
        println!();
    }

    utils::success(&format!("Displayed {} message(s)", messages.len()));

    Ok(())
}

/// List all messages in the system
pub async fn list_all_messages() -> ForgeResult<()> {
    let current_dir = env::current_dir()?;
    let swarm_root = current_dir.join(".forge-swarm");

    if !swarm_root.exists() {
        return Err(ForgeError::Swarm(
            "No swarm found. Run 'forge swarm init' first.".to_string(),
        ));
    }

    let messages = load_all_messages(&swarm_root)?;

    if messages.is_empty() {
        println!("No messages in the swarm");
        return Ok(());
    }

    utils::header("All Swarm Messages");
    println!();

    for message in &messages {
        let type_badge = format_message_type(&message.message_type);
        let from_badge = format_from_badge(&message.from);

        let to_badge = if message.to == "all" {
            "[BROADCAST]".yellow().to_string()
        } else {
            format!("[TO: {}]", message.to).green().to_string()
        };

        println!(
            "{} {} → {} {}",
            type_badge,
            from_badge,
            to_badge,
            message.timestamp.dimmed()
        );
        println!("  {}", message.content);
        println!();
    }

    utils::success(&format!("Displayed {} message(s)", messages.len()));

    Ok(())
}
