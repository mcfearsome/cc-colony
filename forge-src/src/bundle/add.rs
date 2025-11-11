use crate::api::ApiClient;
use crate::config::ClaudeConfig;
use crate::error::ForgeResult;
use crate::utils;

pub async fn run(server_name: String) -> ForgeResult<()> {
    let spinner = utils::spinner(&format!("Fetching server '{}'...", server_name));

    let client = ApiClient::new()?;
    let server_info = client.get_server(&server_name).await?;

    spinner.finish_and_clear();

    // Load current config
    let mut config = ClaudeConfig::load()?;

    // Check if server already exists
    if config.has_server(&server_name) {
        utils::warning(&format!("Server '{}' is already installed", server_name));

        if !utils::confirm("Do you want to overwrite it?") {
            utils::info("Installation cancelled");
            return Ok(());
        }
    }

    // Show server details
    utils::header(&format!("Installing '{}'", server_name));
    println!("  Description: {}", server_info.description);
    println!("  Version:     {}", server_info.version);
    println!("  Command:     {}", server_info.command);

    if !server_info.args.is_empty() {
        println!("  Args:        {}", server_info.args.join(" "));
    }

    if !server_info.env.is_empty() {
        println!("  Env vars:    {}", server_info.env.len());
    }

    println!();

    if !utils::confirm("Continue with installation?") {
        utils::info("Installation cancelled");
        return Ok(());
    }

    let spinner = utils::spinner("Installing...");

    // Add server to config
    config.add_server(server_name.clone(), server_info.to_mcp_server());
    config.save()?;

    spinner.finish_and_clear();

    utils::success(&format!("Server '{}' installed successfully!", server_name));
    utils::info("Restart Claude Desktop for changes to take effect");

    Ok(())
}
