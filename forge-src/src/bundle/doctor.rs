use crate::config::ClaudeConfig;
use crate::error::ForgeResult;
use crate::utils;
use colored::Colorize;
use std::path::Path;

pub async fn run() -> ForgeResult<()> {
    utils::header("Running diagnostics...");

    let mut issues_found = 0;
    let mut checks_passed = 0;

    // Check 1: Claude Desktop config exists
    print!("Checking Claude Desktop config... ");
    let config_path = ClaudeConfig::get_config_path()?;
    if config_path.exists() {
        println!("{}", "✓".green());
        checks_passed += 1;
    } else {
        println!("{}", "✗".red());
        utils::error(&format!(
            "Config file not found at {}",
            config_path.display()
        ));
        issues_found += 1;
    }

    // Check 2: Config is valid JSON
    print!("Validating config structure... ");
    match ClaudeConfig::load() {
        Ok(config) => {
            println!("{}", "✓".green());
            checks_passed += 1;

            // Check 3: Validate server configurations
            print!("Checking server configurations... ");
            match config.validate() {
                Ok(validation_issues) => {
                    if validation_issues.is_empty() {
                        println!("{}", "✓".green());
                        checks_passed += 1;
                    } else {
                        println!("{}", "⚠".yellow());
                        let issue_count = validation_issues.len();
                        for issue in validation_issues {
                            utils::warning(&issue);
                        }
                        issues_found += issue_count;
                    }
                }
                Err(e) => {
                    println!("{}", "✗".red());
                    utils::error(&format!("Validation failed: {}", e));
                    issues_found += 1;
                }
            }

            // Check 4: Verify commands exist
            print!("Verifying server commands... ");
            let mut command_issues = 0;

            for (name, server) in &config.mcpServers {
                // Check if command looks like an absolute path
                if (server.command.starts_with('/') || server.command.starts_with("./"))
                    && !Path::new(&server.command).exists()
                {
                    utils::warning(&format!(
                        "Server '{}': command '{}' not found",
                        name, server.command
                    ));
                    command_issues += 1;
                }
                // For non-path commands, we could check PATH but that's OS-specific
                // Skip for now to avoid false positives
            }

            if command_issues == 0 {
                println!("{}", "✓".green());
                checks_passed += 1;
            } else {
                println!("{}", "⚠".yellow());
                issues_found += command_issues;
            }
        }
        Err(e) => {
            println!("{}", "✗".red());
            utils::error(&format!("Failed to load config: {}", e));
            issues_found += 1;
        }
    }

    // Check 5: Forge config
    print!("Checking forge configuration... ");
    match crate::config::ForgeConfig::load() {
        Ok(_) => {
            println!("{}", "✓".green());
            checks_passed += 1;
        }
        Err(e) => {
            println!("{}", "⚠".yellow());
            utils::warning(&format!("Forge config issue: {}", e));
            issues_found += 1;
        }
    }

    // Summary
    println!("\n{}", "Summary".bold().underline());
    println!("  Checks passed: {}", checks_passed.to_string().green());

    if issues_found > 0 {
        println!("  Issues found:  {}", issues_found.to_string().yellow());
        utils::info("Run 'forge doctor' again after fixing issues");
    } else {
        utils::success("No issues found! Everything looks good.");
    }

    Ok(())
}
