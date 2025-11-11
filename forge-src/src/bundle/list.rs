use crate::config::ClaudeConfig;
use crate::error::ForgeResult;
use crate::utils;
use colored::Colorize;

pub async fn run() -> ForgeResult<()> {
    let config = ClaudeConfig::load()?;

    let servers = config.list_servers();

    if servers.is_empty() {
        utils::info("No MCP servers installed");
        println!(
            "\nRun {} to search for available servers",
            "forge search".cyan()
        );
        return Ok(());
    }

    utils::header(&format!("Installed MCP Servers ({})", servers.len()));

    let mut sorted_servers = servers;
    sorted_servers.sort();

    for name in sorted_servers {
        if let Some(server) = config.get_server(&name) {
            println!("\n{}", utils::format_server_name(&name));
            println!("  Command: {}", server.command.dimmed());

            if !server.args.is_empty() {
                println!("  Args:    {}", server.args.join(" ").dimmed());
            }

            if !server.env.is_empty() {
                println!("  Env:     {} variable(s)", server.env.len());
            }
        }
    }

    println!();

    Ok(())
}
