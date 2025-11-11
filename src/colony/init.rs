use std::path::Path;

use crate::colony::config::ColonyConfig;
use crate::colony::controller::ColonyController;
use crate::colony::tasks::queue::TaskQueue;
use crate::colony::worktree;
use crate::error::ColonyResult;
use crate::utils;

/// Initialize a new colony configuration
pub async fn run() -> ColonyResult<()> {
    // Check if we're in a Git repository
    if !worktree::is_git_repo() {
        return Err(crate::error::ColonyError::Colony(
            "forge colony must be run inside a Git repository".to_string(),
        ));
    }

    let config_path = Path::new("colony.yml");

    // Check if config already exists
    if config_path.exists() {
        let overwrite = utils::confirm("colony.yml already exists. Overwrite?");
        if !overwrite {
            utils::info("Initialization cancelled");
            return Ok(());
        }
    }

    // Create default configuration
    let config = ColonyConfig::default();

    // Save to file
    config.save(config_path)?;

    // Initialize task queue directory structure
    let controller = ColonyController::new(config.clone())?;
    let task_queue = TaskQueue::new(controller.colony_root());
    task_queue.initialize()?;

    utils::success("Created colony.yml");
    utils::success("Initialized task queue directories");
    utils::info("\nNext steps:");
    println!("  1. Edit colony.yml to configure your agents");
    println!("  2. Run 'forge colony start' to spawn agents");
    println!("  3. Use 'forge colony status' to monitor progress");

    // Show the generated config
    println!("\nGenerated configuration:");
    println!("{}", "─".repeat(50));
    let yaml = std::fs::read_to_string(config_path)?;
    println!("{}", yaml);
    println!("{}", "─".repeat(50));

    Ok(())
}
