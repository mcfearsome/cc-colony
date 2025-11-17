//! Health check command for Colony system

use crate::colony::ColonyConfig;
use crate::error::{ColonyError, ColonyResult};
use crate::utils;
use colored::Colorize;
use std::path::Path;

/// Run health checks on the colony system
pub async fn run() -> ColonyResult<()> {
    utils::header("Colony Health Check");

    let mut all_healthy = true;

    // Check 1: Colony configuration
    print!("  {} Colony configuration... ", "◆".cyan());
    match check_config().await {
        Ok(()) => {
            println!("{}", "✓ OK".green());
        }
        Err(e) => {
            println!("{}", format!("✗ FAILED: {}", e).red());
            all_healthy = false;
        }
    }

    // Check 2: Colony directory structure
    print!("  {} Colony directory... ", "◆".cyan());
    match check_colony_directory() {
        Ok(()) => {
            println!("{}", "✓ OK".green());
        }
        Err(e) => {
            println!("{}", format!("✗ FAILED: {}", e).red());
            all_healthy = false;
        }
    }

    // Check 3: Git repository
    print!("  {} Git repository... ", "◆".cyan());
    match check_git_repository() {
        Ok(status) => {
            println!("{}", format!("✓ OK ({})", status).green());
        }
        Err(e) => {
            println!("{}", format!("⚠ WARNING: {}", e).yellow());
            // Git not being available is a warning, not a failure
        }
    }

    // Check 4: Tmux availability
    print!("  {} Tmux installation... ", "◆".cyan());
    match check_tmux() {
        Ok(()) => {
            println!("{}", "✓ OK".green());
        }
        Err(e) => {
            println!("{}", format!("✗ FAILED: {}", e).red());
            all_healthy = false;
        }
    }

    // Check 5: Shared state (if configured)
    print!("  {} Shared state... ", "◆".cyan());
    match check_shared_state().await {
        Ok(Some(status)) => {
            println!("{}", format!("✓ OK ({})", status).green());
        }
        Ok(None) => {
            println!("{}", "○ Not configured".dimmed());
        }
        Err(e) => {
            println!("{}", format!("⚠ WARNING: {}", e).yellow());
        }
    }

    // Check 6: Agent state
    print!("  {} Agent state... ", "◆".cyan());
    match check_agent_state() {
        Ok((running, total)) => {
            if running > 0 {
                println!(
                    "{}",
                    format!("✓ {} of {} agents running", running, total).green()
                );
            } else if total > 0 {
                println!(
                    "{}",
                    format!("○ 0 of {} agents running (colony not started)", total).dimmed()
                );
            } else {
                println!("{}", "○ No agents configured".dimmed());
            }
        }
        Err(e) => {
            println!("{}", format!("⚠ WARNING: {}", e).yellow());
        }
    }

    // Check 7: Message queue
    print!("  {} Message queue... ", "◆".cyan());
    match check_message_queue() {
        Ok(count) => {
            if count > 0 {
                println!(
                    "{}",
                    format!("✓ OK ({} messages pending)", count).green()
                );
            } else {
                println!("{}", "✓ OK (empty)".green());
            }
        }
        Err(e) => {
            println!("{}", format!("⚠ WARNING: {}", e).yellow());
        }
    }

    println!();

    if all_healthy {
        utils::success("All critical systems healthy");
        Ok(())
    } else {
        utils::warning("Some critical systems are not healthy");
        Err(ColonyError::Colony(
            "Health check failed. See above for details.".to_string(),
        ))
    }
}

/// Check colony configuration
async fn check_config() -> ColonyResult<()> {
    let config_path = Path::new("colony.yml");

    if !config_path.exists() {
        return Err(ColonyError::Colony(
            "colony.yml not found".to_string(),
        ));
    }

    let config = ColonyConfig::load(config_path)?;
    config.validate()?;

    Ok(())
}

/// Check colony directory structure
fn check_colony_directory() -> ColonyResult<()> {
    let colony_root = Path::new(".colony");

    if !colony_root.exists() {
        return Err(ColonyError::Colony(
            ".colony directory not found. Run 'colony init' or 'colony start'.".to_string(),
        ));
    }

    // Check for essential subdirectories
    let messages_dir = colony_root.join("messages");
    if !messages_dir.exists() {
        return Err(ColonyError::Colony(
            "messages directory missing".to_string(),
        ));
    }

    Ok(())
}

/// Check git repository status
fn check_git_repository() -> Result<String, String> {
    // Check if we're in a git repo
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .map_err(|e| format!("Git not available: {}", e))?;

    if !output.status.success() {
        return Err("Not a git repository".to_string());
    }

    // Get current branch
    let output = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .output()
        .map_err(|e| format!("Failed to get branch: {}", e))?;

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Check if there are uncommitted changes
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map_err(|e| format!("Failed to check status: {}", e))?;

    let has_changes = !output.stdout.is_empty();

    if has_changes {
        Ok(format!("branch: {}, uncommitted changes", branch))
    } else {
        Ok(format!("branch: {}, clean", branch))
    }
}

/// Check tmux installation
fn check_tmux() -> ColonyResult<()> {
    let output = std::process::Command::new("tmux")
        .args(["-V"])
        .output();

    match output {
        Ok(o) if o.status.success() => Ok(()),
        Ok(_) => Err(ColonyError::Colony("Tmux check failed".to_string())),
        Err(_) => Err(ColonyError::Colony(
            "Tmux not installed. Install with: brew install tmux (macOS) or apt-get install tmux (Linux)".to_string(),
        )),
    }
}

/// Check shared state system
async fn check_shared_state() -> Result<Option<String>, String> {
    let config_path = Path::new("colony.yml");

    if !config_path.exists() {
        return Ok(None);
    }

    let config = ColonyConfig::load(config_path)
        .map_err(|e| format!("Failed to load config: {}", e))?;

    if config.shared_state.is_none() {
        return Ok(None);
    }

    let state_dir = Path::new(".colony/state");

    if !state_dir.exists() {
        return Err("State directory not initialized".to_string());
    }

    // Check for state files
    let tasks_file = state_dir.join("tasks.jsonl");
    let workflows_file = state_dir.join("workflows.jsonl");

    let mut files = Vec::new();
    if tasks_file.exists() {
        files.push("tasks");
    }
    if workflows_file.exists() {
        files.push("workflows");
    }

    if files.is_empty() {
        Ok(Some("initialized, no data yet".to_string()))
    } else {
        Ok(Some(format!("files: {}", files.join(", "))))
    }
}

/// Check agent state
fn check_agent_state() -> Result<(usize, usize), String> {
    let config_path = Path::new("colony.yml");

    if !config_path.exists() {
        return Err("No configuration".to_string());
    }

    let config = ColonyConfig::load(config_path)
        .map_err(|e| format!("Failed to load config: {}", e))?;

    let total = config.agents.len();

    // Check state file
    let state_path = Path::new(".colony/state.json");

    if !state_path.exists() {
        return Ok((0, total));
    }

    let content = std::fs::read_to_string(state_path)
        .map_err(|e| format!("Failed to read state: {}", e))?;

    let states: Vec<serde_json::Value> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse state: {}", e))?;

    let running = states
        .iter()
        .filter(|s| s.get("status").and_then(|v| v.as_str()) == Some("running"))
        .count();

    Ok((running, total))
}

/// Check message queue
fn check_message_queue() -> Result<usize, String> {
    let messages_dir = Path::new(".colony/messages");

    if !messages_dir.exists() {
        return Err("Messages directory not found".to_string());
    }

    // Count pending messages across all inboxes
    let mut total = 0;

    if let Ok(entries) = std::fs::read_dir(messages_dir) {
        for entry in entries.flatten() {
            if entry.file_type().ok().map_or(false, |t| t.is_dir()) {
                let inbox_path = entry.path();
                if let Ok(messages) = std::fs::read_dir(&inbox_path) {
                    total += messages.count();
                }
            }
        }
    }

    Ok(total)
}
