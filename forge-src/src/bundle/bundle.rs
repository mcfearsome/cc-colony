use crate::api::ApiClient;
use crate::config::ClaudeConfig;
use crate::error::ForgeResult;
use crate::utils;
use colored::Colorize;

pub async fn list() -> ForgeResult<()> {
    let spinner = utils::spinner("Fetching available bundles...");

    let client = ApiClient::new()?;
    let bundles = client.list_bundles().await?;

    spinner.finish_and_clear();

    if bundles.is_empty() {
        utils::info("No bundles available");
        return Ok(());
    }

    utils::header(&format!("Available Bundles ({})", bundles.len()));

    for bundle in bundles {
        println!("\n{}", utils::format_server_name(&bundle.name));
        println!("  {}", bundle.description.dimmed());
        println!("  Servers: {} MCP server(s)", bundle.servers.len());

        if !bundle.tags.is_empty() {
            print!("  Tags: ");
            for tag in &bundle.tags {
                print!("{} ", utils::format_tag(tag));
            }
            println!();
        }
    }

    println!(
        "\nRun {} to activate a bundle",
        "forge bundle activate <name>".cyan()
    );

    Ok(())
}

pub async fn activate(name: String) -> ForgeResult<()> {
    let spinner = utils::spinner(&format!("Fetching bundle '{}'...", name));

    let client = ApiClient::new()?;
    let bundle = client.get_bundle(&name).await?;

    spinner.finish_and_clear();

    utils::header(&format!("Activating bundle '{}'", name));
    println!("  Description: {}", bundle.description);
    println!("  Servers:     {} MCP server(s)", bundle.servers.len());
    println!();

    // Show servers that will be installed
    println!("The following servers will be installed:");
    for server_name in &bundle.servers {
        println!("  • {}", server_name.cyan());
    }
    println!();

    if !utils::confirm("Continue with bundle activation?") {
        utils::info("Bundle activation cancelled");
        return Ok(());
    }

    let mut config = ClaudeConfig::load()?;

    let pb = utils::progress_bar(bundle.servers.len() as u64, "Installing servers");

    for server_name in &bundle.servers {
        pb.set_message(format!("Installing '{}'...", server_name));

        match client.get_server(server_name).await {
            Ok(server_info) => {
                config.add_server(server_name.clone(), server_info.to_mcp_server());
            }
            Err(e) => {
                utils::warning(&format!("Failed to install '{}': {}", server_name, e));
            }
        }

        pb.inc(1);
    }

    pb.finish_with_message("Installation complete");

    config.save()?;

    utils::success(&format!("Bundle '{}' activated successfully!", name));
    utils::info("Restart Claude Desktop for changes to take effect");

    Ok(())
}
