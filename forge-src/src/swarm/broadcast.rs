use std::path::Path;

use crate::error::ForgeResult;
use crate::swarm::{messaging, SwarmConfig, SwarmController};
use crate::utils;

/// Broadcast a message to all agents
pub async fn run(message: String) -> ForgeResult<()> {
    let config_path = Path::new("swarm.yml");

    if !config_path.exists() {
        return Err(crate::error::ForgeError::Swarm(
            "swarm.yml not found. Run 'forge swarm init' first.".to_string(),
        ));
    }

    // Load configuration
    let config = SwarmConfig::load(config_path)?;
    let session_name = config.session_name();

    // Create controller
    let controller = SwarmController::new(config)?;

    // Create broadcast message
    let msg = messaging::Message::new(
        "operator",
        "all",
        message.clone(),
        messaging::MessageType::Info,
    );

    // Save message
    msg.save(controller.swarm_root())?;

    utils::success("Broadcast message sent to all agents");
    println!("\nMessage: {}", message);

    // Also display in tmux if session exists
    if crate::swarm::tmux::session_exists(&session_name) {
        // Escape tmux format sequences (# -> ##) to prevent format string injection
        let escaped_msg = message.replace('#', "##");
        let tmux_msg = format!("📢 BROADCAST: {}", escaped_msg);
        let _ = std::process::Command::new("tmux")
            .arg("display-message")
            .arg("-t")
            .arg(&session_name)
            .arg(&tmux_msg)
            .output();

        utils::info("Message displayed in tmux session");
    }

    Ok(())
}
