use std::path::Path;

use crate::error::ForgeResult;
use crate::swarm::config::SwarmConfig;
use crate::swarm::controller::SwarmController;
use crate::swarm::tasks::queue::TaskQueue;
use crate::swarm::worktree;
use crate::utils;

/// Initialize a new swarm configuration
pub async fn run() -> ForgeResult<()> {
    // Check if we're in a Git repository
    if !worktree::is_git_repo() {
        return Err(crate::error::ForgeError::Swarm(
            "forge swarm must be run inside a Git repository".to_string(),
        ));
    }

    let config_path = Path::new("swarm.yml");

    // Check if config already exists
    if config_path.exists() {
        let overwrite = utils::confirm("swarm.yml already exists. Overwrite?");
        if !overwrite {
            utils::info("Initialization cancelled");
            return Ok(());
        }
    }

    // Create default configuration
    let config = SwarmConfig::default();

    // Save to file
    config.save(config_path)?;

    // Initialize task queue directory structure
    let controller = SwarmController::new(config.clone())?;
    let task_queue = TaskQueue::new(controller.swarm_root());
    task_queue.initialize()?;

    utils::success("Created swarm.yml");
    utils::success("Initialized task queue directories");
    utils::info("\nNext steps:");
    println!("  1. Edit swarm.yml to configure your agents");
    println!("  2. Run 'forge swarm start' to spawn agents");
    println!("  3. Use 'forge swarm status' to monitor progress");

    // Show the generated config
    println!("\nGenerated configuration:");
    println!("{}", "─".repeat(50));
    let yaml = std::fs::read_to_string(config_path)?;
    println!("{}", yaml);
    println!("{}", "─".repeat(50));

    Ok(())
}
