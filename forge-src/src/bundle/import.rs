use crate::config::ClaudeConfig;
use crate::error::ForgeResult;
use crate::utils;
use std::fs;

pub async fn run(file: String) -> ForgeResult<()> {
    utils::header(&format!("Importing configuration from '{}'", file));

    // Check if file exists
    if !std::path::Path::new(&file).exists() {
        utils::error(&format!("File '{}' not found", file));
        return Ok(());
    }

    // Load and validate the import file
    let content = fs::read_to_string(&file)?;
    let import_config: ClaudeConfig = serde_json::from_str(&content)?;

    println!("Import preview:");
    println!("  Servers: {}", import_config.mcpServers.len());

    for (name, server) in &import_config.mcpServers {
        println!("    • {} ({})", name, server.command);
    }

    println!();

    // Load current config
    let current_config = ClaudeConfig::load()?;

    if !current_config.mcpServers.is_empty() {
        utils::warning(&format!(
            "Current configuration has {} server(s) that will be replaced",
            current_config.mcpServers.len()
        ));
    }

    if !utils::confirm("Continue with import?") {
        utils::info("Import cancelled");
        return Ok(());
    }

    let spinner = utils::spinner("Importing...");

    // Save the imported config
    import_config.save()?;

    spinner.finish_and_clear();

    utils::success("Configuration imported successfully!");
    utils::info("Restart Claude Desktop for changes to take effect");

    Ok(())
}
