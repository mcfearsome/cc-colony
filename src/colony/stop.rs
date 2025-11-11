use std::path::Path;

use crate::colony::{AgentStatus, ColonyConfig, ColonyController};
use crate::error::ColonyResult;
use crate::utils;

/// Stop one or all agents
pub async fn run(agent_id: Option<String>) -> ColonyResult<()> {
    let config_path = Path::new("colony.yml");

    if !config_path.exists() {
        return Err(crate::error::ColonyError::Colony(
            "colony.yml not found. Run 'colony init' first.".to_string(),
        ));
    }

    // Load configuration
    let config = ColonyConfig::load(config_path)?;

    // Create controller
    let mut controller = ColonyController::new(config)?;
    controller.initialize_agents()?;

    // Load state
    controller.load_state()?;

    match agent_id {
        Some(id) => {
            // Stop specific agent
            utils::info(&format!("Stopping agent: {}", id));
            stop_agent(&mut controller, &id).await?;
        }
        None => {
            // Stop all agents
            utils::header("Stopping all agents");
            let agent_ids: Vec<String> = controller.agents().keys().cloned().collect();

            for id in agent_ids {
                stop_agent(&mut controller, &id).await?;
            }
        }
    }

    // Save state
    controller.save_state()?;

    utils::success("Stopped agents");

    Ok(())
}

async fn stop_agent(controller: &mut ColonyController, agent_id: &str) -> ColonyResult<()> {
    let agent = controller
        .get_agent_mut(agent_id)
        .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

    // Check if agent has a PID
    if let Some(pid) = agent.pid {
        // Validate PID is reasonable (max PID on Linux is typically 4194304)
        if pid == 0 || pid > 4_194_304 {
            utils::warning(&format!(
                "  Invalid PID {} for agent '{}' - cannot stop",
                pid, agent_id
            ));
        } else if !is_process_running(pid) {
            utils::info(&format!(
                "  Process {} for agent '{}' is not running",
                pid, agent_id
            ));
        } else {
            // Try to kill the process
            #[cfg(unix)]
            {
                use std::process::Command;
                let result = Command::new("kill").arg(pid.to_string()).output();

                match result {
                    Ok(output) if output.status.success() => {
                        utils::success(&format!("  Stopped agent '{}' (PID: {})", agent_id, pid));
                    }
                    Ok(output) => {
                        utils::warning(&format!(
                            "  Failed to stop agent '{}' (PID: {}): {}",
                            agent_id,
                            pid,
                            String::from_utf8_lossy(&output.stderr)
                        ));
                    }
                    Err(e) => {
                        utils::warning(&format!(
                            "  Failed to execute kill command for agent '{}' (PID: {}): {}",
                            agent_id, pid, e
                        ));
                    }
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

                match result {
                    Ok(output) if output.status.success() => {
                        utils::success(&format!("  Stopped agent '{}' (PID: {})", agent_id, pid));
                    }
                    Ok(output) => {
                        utils::warning(&format!(
                            "  Failed to stop agent '{}' (PID: {}): {}",
                            agent_id,
                            pid,
                            String::from_utf8_lossy(&output.stderr)
                        ));
                    }
                    Err(e) => {
                        utils::warning(&format!(
                            "  Failed to execute taskkill command for agent '{}' (PID: {}): {}",
                            agent_id, pid, e
                        ));
                    }
                }
            }
        }
    } else {
        utils::info(&format!("  Agent '{}' is not running", agent_id));
    }

    // Update agent state
    agent.set_status(AgentStatus::Idle);
    agent.pid = None;
    agent.process = None;

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
