use crate::config::{ClaudeConfig, ForgeConfig};
use crate::error::ForgeResult;
use crate::utils;
use colored::Colorize;
use std::fs;

pub async fn run() -> ForgeResult<()> {
    utils::header("Initializing Forge");

    let spinner = utils::spinner("Setting up forge configuration...");

    // Create .forge directory
    let forge_dir = std::env::current_dir()?.join(".forge");
    fs::create_dir_all(&forge_dir)?;

    // Check if Claude Desktop config exists
    let claude_config_path = ClaudeConfig::get_config_path()?;
    if !claude_config_path.exists() {
        spinner.finish_with_message("Creating Claude Desktop config...");
        utils::warning(&format!(
            "Claude Desktop config not found at {}",
            claude_config_path.display()
        ));
        utils::info("Creating a new config file...");

        let config = ClaudeConfig::new();
        config.save()?;
    } else {
        spinner.finish_with_message("Claude Desktop config found");
    }

    // Initialize forge config
    let forge_config = ForgeConfig::new();
    forge_config.save()?;

    utils::success("Forge initialized successfully!");
    utils::info(&format!(
        "Configuration directory: {}",
        ForgeConfig::get_config_path()?.parent().unwrap().display()
    ));
    utils::info(&format!(
        "Claude Desktop config: {}",
        claude_config_path.display()
    ));

    println!("\nNext steps:");
    println!(
        "  • Run {} to authenticate with useforge.cc",
        "forge login".cyan()
    );
    println!(
        "  • Run {} to search for MCP servers",
        "forge search".cyan()
    );
    println!("  • Run {} to see installed servers", "forge list".cyan());

    Ok(())
}
