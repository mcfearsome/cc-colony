use std::path::Path;

use crate::error::ColonyResult;
use crate::colony::{ColonyConfig, ColonyController};
use crate::utils;

/// Destroy the colony and clean up all resources
pub async fn run() -> ColonyResult<()> {
    let config_path = Path::new("colony.yml");

    if !config_path.exists() {
        return Err(crate::error::ColonyError::Colony(
            "colony.yml not found. Nothing to destroy.".to_string(),
        ));
    }

    // Confirm destruction
    utils::warning("This will stop all agents and remove all worktrees and state!");
    let confirm = utils::confirm("Are you sure you want to destroy the colony?");

    if !confirm {
        utils::info("Destruction cancelled");
        return Ok(());
    }

    utils::header("Destroying Forge Colony");

    // Load configuration
    let config = ColonyConfig::load(config_path)?;

    // Create controller
    let mut controller = ColonyController::new(config)?;
    controller.initialize_agents()?;

    // Load state
    let _ = controller.load_state();

    // Stop all running agents
    let spinner = utils::spinner("Stopping all agents...");
    for agent in controller.agents_mut().values_mut() {
        if let Some(pid) = agent.pid {
            // Validate PID is reasonable (max PID on Linux is typically 4194304)
            if pid == 0 || pid > 4_194_304 {
                utils::warning(&format!(
                    "  Invalid PID {} for agent '{}' - skipping",
                    pid,
                    agent.id()
                ));
                continue;
            }

            if !is_process_running(pid) {
                continue; // Process already stopped
            }

            // Try to kill the process
            #[cfg(unix)]
            {
                use std::process::Command;
                let result = Command::new("kill").arg(pid.to_string()).output();

                if let Err(e) = result {
                    utils::warning(&format!(
                        "  Failed to execute kill command for agent '{}' (PID: {}): {}",
                        agent.id(),
                        pid,
                        e
                    ));
                }
            }

            #[cfg(windows)]
            {
                use std::process::Command;
                let result = Command::new("taskkill")
                    .arg("/F")
                    .arg("/PID")
                    .arg(pid.to_string())
                    .output();

                if let Err(e) = result {
                    utils::warning(&format!(
                        "  Failed to execute taskkill command for agent '{}' (PID: {}): {}",
                        agent.id(),
                        pid,
                        e
                    ));
                }
            }
        }
    }
    spinner.finish_and_clear();
    utils::success("Stopped all agents");

    // Remove all worktrees
    let spinner = utils::spinner("Removing Git worktrees...");
    if let Err(e) = controller.cleanup_worktrees() {
        spinner.finish_and_clear();
        utils::warning(&format!("Failed to clean up some worktrees: {}", e));
    } else {
        spinner.finish_and_clear();
        utils::success("Removed Git worktrees");
    }

    // Remove colony directory
    let colony_root = controller.colony_root();
    if colony_root.exists() {
        let spinner = utils::spinner("Removing colony directory...");
        std::fs::remove_dir_all(colony_root)?;
        spinner.finish_and_clear();
        utils::success("Removed colony directory");
    }

    utils::success("Colony destroyed successfully");
    utils::info("\nThe colony.yml file has been preserved.");
    utils::info("Run 'forge colony start' to recreate the colony.");

    Ok(())
}

/// Check if a process is running
fn is_process_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        use std::process::Command;
        // Use kill -0 to check if process exists without sending a signal
        Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    #[cfg(windows)]
    {
        use std::process::Command;
        // Use tasklist to check if process exists
        Command::new("tasklist")
            .arg("/FI")
            .arg(format!("PID eq {}", pid))
            .output()
            .map(|output| {
                output.status.success()
                    && String::from_utf8_lossy(&output.stdout).contains(&pid.to_string())
            })
            .unwrap_or(false)
    }

    #[cfg(not(any(unix, windows)))]
    {
        // On unsupported platforms, assume it's running to be safe
        true
    }
}
