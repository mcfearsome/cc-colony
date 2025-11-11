use crate::config::ClaudeConfig;
use crate::error::ForgeResult;
use crate::utils;

pub async fn run(server_name: String) -> ForgeResult<()> {
    let mut config = ClaudeConfig::load()?;

    // Check if server exists
    if !config.has_server(&server_name) {
        utils::error(&format!("Server '{}' is not installed", server_name));
        utils::info("Run 'forge list' to see installed servers");
        return Ok(());
    }

    utils::header(&format!("Removing '{}'", server_name));

    if !utils::confirm(&format!(
        "Are you sure you want to remove '{}'?",
        server_name
    )) {
        utils::info("Removal cancelled");
        return Ok(());
    }

    let spinner = utils::spinner("Removing...");

    config.remove_server(&server_name)?;
    config.save()?;

    spinner.finish_and_clear();

    utils::success(&format!("Server '{}' removed successfully!", server_name));
    utils::info("Restart Claude Desktop for changes to take effect");

    Ok(())
}
