use crate::api::ApiClient;
use crate::error::ForgeResult;
use crate::utils;
use colored::Colorize;

pub async fn run(query: Option<String>) -> ForgeResult<()> {
    let spinner = utils::spinner("Searching for MCP servers...");

    let client = ApiClient::new()?;
    let servers = client.search_servers(query.clone()).await?;

    spinner.finish_and_clear();

    if servers.is_empty() {
        utils::info("No servers found");
        return Ok(());
    }

    let query_str = query.unwrap_or_else(|| "all servers".to_string());
    utils::header(&format!(
        "Search results for '{}' ({} found)",
        query_str,
        servers.len()
    ));

    for server in servers {
        println!(
            "\n{} {}",
            utils::format_server_name(&server.name),
            utils::format_version(&server.version)
        );
        println!("  {}", server.description.dimmed());

        if !server.tags.is_empty() {
            print!("  Tags: ");
            for tag in &server.tags {
                print!("{} ", utils::format_tag(tag));
            }
            println!();
        }

        if let Some(author) = &server.author {
            println!("  Author: {}", author.dimmed());
        }

        if let Some(repo) = &server.repository {
            println!("  Repo: {}", repo.blue().underline());
        }
    }

    println!(
        "\nRun {} to install a server",
        "forge add <server-name>".cyan()
    );

    Ok(())
}
