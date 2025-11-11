use std::path::Path;

use crate::colony::{tmux, ColonyConfig};
use crate::error::ColonyResult;
use crate::utils;

/// Attach to an existing tmux session
pub async fn run() -> ColonyResult<()> {
    let config_path = Path::new("colony.yml");

    if !config_path.exists() {
        return Err(crate::error::ColonyError::Colony(
            "colony.yml not found. Run 'colony init' first.".to_string(),
        ));
    }

    // Check if tmux is available
    if !tmux::is_tmux_available() {
        return Err(crate::error::ColonyError::Colony(
            "tmux is not installed. Please install tmux to use this feature.".to_string(),
        ));
    }

    // Load configuration to get session name
    let config = ColonyConfig::load(config_path)?;
    let session_name = config.session_name();

    // Check if session exists
    if !tmux::session_exists(&session_name) {
        return Err(crate::error::ColonyError::Colony(format!(
            "No tmux session '{}' found. Start the colony with 'colony start' first.",
            session_name
        )));
    }

    utils::info(&format!("Attaching to tmux session '{}'...", session_name));
    utils::info("Press Ctrl+B then D to detach from the session");

    // Attach to the session
    tmux::attach_session(&session_name)?;

    Ok(())
}
