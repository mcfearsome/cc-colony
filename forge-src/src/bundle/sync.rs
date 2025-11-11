use crate::api::ApiClient;
use crate::config::ClaudeConfig;
use crate::error::ForgeResult;
use crate::utils;
use colored::Colorize;

pub async fn run() -> ForgeResult<()> {
    let spinner = utils::spinner("Syncing with useforge.cc...");

    let client = ApiClient::new()?;
    let sync_response = client.sync().await?;

    spinner.finish_and_clear();

    utils::header("Sync Results");

    // Show recommended servers
    if !sync_response.recommended_servers.is_empty() {
        println!("\n{}", "Recommended Servers:".bold());
        for server in &sync_response.recommended_servers {
            println!(
                "  • {} - {}",
                utils::format_server_name(&server.name),
                server.description.dimmed()
            );
        }
    }

    // Show recommended bundles
    if !sync_response.recommended_bundles.is_empty() {
        println!("\n{}", "Recommended Bundles:".bold());
        for bundle in &sync_response.recommended_bundles {
            println!(
                "  • {} - {}",
                utils::format_server_name(&bundle.name),
                bundle.description.dimmed()
            );
        }
    }

    // Show available updates
    if !sync_response.updates.is_empty() {
        println!("\n{}", "Available Updates:".bold());
        for update in &sync_response.updates {
            println!(
                "  • {} {} → {}",
                update.server_name.cyan(),
                update.current_version.dimmed(),
                utils::format_version(&update.latest_version)
            );

            if let Some(changelog) = &update.changelog {
                println!("    {}", changelog.dimmed());
            }
        }

        println!();

        if utils::confirm("Install available updates?") {
            let config = ClaudeConfig::load()?;
            let pb = utils::progress_bar(sync_response.updates.len() as u64, "Updating servers");

            let mut updated_config = config;

            for update in &sync_response.updates {
                pb.set_message(format!("Updating '{}'...", update.server_name));

                match client.get_server(&update.server_name).await {
                    Ok(server_info) => {
                        updated_config
                            .add_server(update.server_name.clone(), server_info.to_mcp_server());
                    }
                    Err(e) => {
                        utils::warning(&format!(
                            "Failed to update '{}': {}",
                            update.server_name, e
                        ));
                    }
                }

                pb.inc(1);
            }

            pb.finish_with_message("Updates complete");
            updated_config.save()?;

            utils::success("All servers updated!");
            utils::info("Restart Claude Desktop for changes to take effect");
        }
    } else {
        utils::success("All servers are up to date!");
    }

    Ok(())
}
