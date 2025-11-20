use std::path::Path;
use std::collections::HashMap;

use crate::colony::config::{AgentConfig, ColonyConfig, ExecutorConfig, McpServerConfig};
use crate::colony::controller::ColonyController;
use crate::colony::mcp_registry::McpRegistry;
use crate::colony::tasks::queue::TaskQueue;
use crate::colony::worktree;
use crate::error::ColonyResult;
use crate::utils;
use colored::Colorize;

/// Initialize a new colony configuration with interactive wizard
pub async fn run() -> ColonyResult<()> {
    // Check if we're in a Git repository
    if !worktree::is_git_repo() {
        return Err(crate::error::ColonyError::Colony(
            "colony must be run inside a Git repository".to_string(),
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

    println!();
    println!("{}", "Colony Configuration Wizard".bold().green());
    println!("{}", "─".repeat(50));
    println!();
    println!("This wizard will help you create a colony.yml configuration.");
    println!("Press Ctrl+C at any time to cancel.");
    println!();

    // Run the interactive wizard
    let config = run_wizard()?;

    // Save to file
    config.save(config_path)?;

    // Initialize task queue directory structure
    let controller = ColonyController::new(config.clone())?;
    let task_queue = TaskQueue::new(controller.colony_root());
    task_queue.initialize()?;

    println!();
    utils::success("Created colony.yml");
    utils::success("Initialized task queue directories");

    println!();
    utils::info("Next steps:");
    println!("  1. Run {} to spawn agents", "colony start".cyan());
    println!("  2. Use {} to monitor progress", "colony status".cyan());
    println!("  3. Use {} for real-time monitoring", "colony tui".cyan());

    // Show the generated config
    println!();
    println!("{}", "Generated Configuration:".bold());
    println!("{}", "─".repeat(50));
    let yaml = std::fs::read_to_string(config_path)?;
    println!("{}", yaml.dimmed());
    println!("{}", "─".repeat(50));

    Ok(())
}

fn run_wizard() -> ColonyResult<ColonyConfig> {
    println!();
    println!("{}", "Choose setup mode:".bold().cyan());
    println!("{}", "─".repeat(50));

    let setup_modes: Vec<String> = vec![
        "Quick Start - Get started fast with smart defaults".to_string(),
        "Custom Setup - Full control over configuration".to_string(),
    ];

    let setup_mode = utils::select("Setup mode", &setup_modes).unwrap_or(0);

    if setup_mode == 0 {
        // Quick start mode
        return run_quick_wizard();
    }

    // Custom setup mode (original wizard)
    // Ask how many agents to create
    let num_agents = utils::prompt_number(
        "How many agents do you want to create?",
        Some(2),
    )
    .unwrap_or(2);

    if num_agents == 0 {
        return Err(crate::error::ColonyError::Colony(
            "You must create at least one agent".to_string(),
        ));
    }

    let mut agents = Vec::new();

    // Template options
    let template_options = vec![
        ("code-reviewer", "Code Reviewer - Quality and best practices review", "Code Reviewer", "Review code for quality, best practices, and potential issues"),
        ("security-auditor", "Security Auditor - OWASP Top 10 security scanning", "Security Auditor", "Identify and document security vulnerabilities"),
        ("test-engineer", "Test Engineer - Automated testing and QA", "Test Engineer", "Write comprehensive tests and improve test coverage"),
        ("documentation-writer", "Documentation Writer - Technical documentation", "Documentation Writer", "Write and maintain technical documentation"),
        ("data-analyst", "Data Analyst - Data analysis and insights", "Data Analyst", "Analyze data and generate insights"),
    ];

    let mut template_choices: Vec<String> = template_options
        .iter()
        .map(|(_, desc, _, _)| desc.to_string())
        .collect();
    template_choices.push("None - I'll configure manually".to_string());

    // Configure each agent
    for i in 0..num_agents {
        println!();
        println!("{}", format!("Agent {} Configuration", i + 1).bold().cyan());
        println!("{}", "─".repeat(50));

        // Agent ID
        let default_id = format!("agent-{}", i + 1);
        let agent_id = utils::prompt(
            "Agent ID (unique identifier)",
            Some(&default_id),
        )
        .unwrap_or(default_id);

        // Ask if they want to use a template
        let template_choice = utils::select(
            "Choose a template (or None for manual configuration)",
            &template_choices,
        );

        let (role, focus, startup_prompt) = match template_choice {
            Some(idx) if idx < template_options.len() => {
                // Using a template
                let (template_name, _, default_role, default_focus) = template_options[idx];

                println!();
                println!("{} Using template: {}", "✓".green(), template_name.cyan());

                // Ask if they want to customize
                let customize_role = utils::confirm("Customize the role?");
                let role = if customize_role {
                    utils::prompt("Agent role", Some(default_role))
                        .unwrap_or_else(|| default_role.to_string())
                } else {
                    default_role.to_string()
                };

                let customize_focus = utils::confirm("Customize the focus?");
                let focus = if customize_focus {
                    utils::prompt("Agent focus", Some(default_focus))
                        .unwrap_or_else(|| default_focus.to_string())
                } else {
                    default_focus.to_string()
                };

                // Ask if they want to add custom startup prompt
                let add_prompt = utils::confirm("Add a custom startup prompt? (will override template)");
                let prompt = if add_prompt {
                    utils::prompt("Custom startup prompt", None)
                } else {
                    None
                };

                (role, focus, prompt)
            }
            _ => {
                // No template, ask for everything
                let role = utils::prompt(
                    "Agent role (e.g., Backend Engineer, Frontend Developer)",
                    Some("Software Developer"),
                )
                .unwrap_or_else(|| "Software Developer".to_string());

                let focus = utils::prompt(
                    "Agent focus (e.g., API endpoints, React components)",
                    Some("General software development"),
                )
                .unwrap_or_else(|| "General software development".to_string());

                let prompt = utils::prompt(
                    "Startup prompt (optional, press Enter to skip)",
                    None,
                );

                (role, focus, prompt)
            }
        };

        // Model selection (optional)
        let default_model = "claude-sonnet-4-20250514";
        let customize_model = utils::confirm("Use a different Claude model?");
        let model = if customize_model {
            let model_options = vec![
                "claude-sonnet-4-20250514 (balanced, recommended)".to_string(),
                "claude-opus-4-20250514 (most capable)".to_string(),
                "claude-3-5-haiku-20241022 (fast and efficient)".to_string(),
            ];
            let model_idx = utils::select("Select Claude model", &model_options)
                .unwrap_or(0);

            match model_idx {
                0 => "claude-sonnet-4-20250514",
                1 => "claude-opus-4-20250514",
                2 => "claude-3-5-haiku-20241022",
                _ => default_model,
            }
        } else {
            default_model
        };

        agents.push(AgentConfig {
            id: agent_id.clone(),
            role,
            focus,
            model: model.to_string(),
            directory: None,
            worktree: Some(format!("agent/{}", agent_id)),
            env: None,
            mcp_servers: None,
            instructions: None,
            startup_prompt,
            capabilities: None,
            nudge: None,
        });
    }

    // Colony name (optional)
    println!();
    let set_name = utils::confirm("Give this colony a name? (optional, uses directory name if not set)");
    let name = if set_name {
        utils::prompt("Colony name", None)
    } else {
        None
    };

    // Telemetry opt-in
    println!();
    println!("{}", "Telemetry".bold().cyan());
    println!("{}", "─".repeat(50));
    println!("Colony can collect anonymous usage data to help improve the tool.");
    println!("This includes:");
    println!("  • Commands used (not arguments or data)");
    println!("  • Feature usage patterns");
    println!("  • Error types (no sensitive information)");
    println!("  • Performance metrics");
    println!();
    println!("{}", "We never collect:".bold());
    println!("  • Code, file contents, or paths");
    println!("  • API keys or credentials");
    println!("  • Personal information");
    println!();

    let enable_telemetry = utils::confirm("Enable telemetry to help improve Colony?");

    let mut telemetry = crate::colony::config::TelemetryConfig::default();
    if enable_telemetry {
        telemetry.enabled = true;
        telemetry.get_or_create_anonymous_id();
        utils::success("Telemetry enabled (you can disable it anytime in colony.yml)");
    } else {
        utils::info("Telemetry disabled (you can enable it anytime in colony.yml)");
    }

    Ok(ColonyConfig {
        name,
        repository: None,
        agents,
        executor: None,
        shared_state: None,
        auth: Default::default(),
        telemetry,
        capabilities: None,
        layout: None,
    })
}

/// Quick wizard with smart defaults based on project type
fn run_quick_wizard() -> ColonyResult<ColonyConfig> {
    println!();
    println!("{}", "Quick Start Wizard".bold().green());
    println!("{}", "─".repeat(50));
    println!("This wizard will set up a colony with smart defaults");
    println!();

    // Ask about project type/purpose
    let project_types: Vec<String> = vec![
        "Web Application - Frontend, backend, and full-stack development".to_string(),
        "CLI Tool - Command-line application development".to_string(),
        "Data Analysis - Research, analytics, and data processing".to_string(),
        "Automation - Scripts, bots, and workflow automation".to_string(),
        "General Development - Mixed or general software development".to_string(),
    ];

    let project_type = utils::select("What type of project are you working on?", &project_types)
        .unwrap_or(4);

    // Configure agents based on project type
    let (agents, default_mcp_servers) = match project_type {
        0 => {
            // Web Application
            (
                vec![
                    AgentConfig {
                        id: "backend".to_string(),
                        role: "Backend Engineer".to_string(),
                        focus: "API endpoints, server logic, and database operations".to_string(),
                        model: "claude-sonnet-4-20250514".to_string(),
                        directory: None,
                        worktree: Some("backend".to_string()),
                        env: None,
                        mcp_servers: None,
                        instructions: None,
                        startup_prompt: None,
                        capabilities: None,
                        nudge: None,
                    },
                    AgentConfig {
                        id: "frontend".to_string(),
                        role: "Frontend Engineer".to_string(),
                        focus: "UI components, styling, and user interactions".to_string(),
                        model: "claude-sonnet-4-20250514".to_string(),
                        directory: None,
                        worktree: Some("frontend".to_string()),
                        env: None,
                        mcp_servers: None,
                        instructions: None,
                        startup_prompt: None,
                        capabilities: None,
                        nudge: None,
                    },
                ],
                McpRegistry::for_web_development(),
            )
        }
        1 => {
            // CLI Tool
            (
                vec![
                    AgentConfig {
                        id: "cli-dev".to_string(),
                        role: "CLI Developer".to_string(),
                        focus: "Command-line interface, argument parsing, and core functionality".to_string(),
                        model: "claude-sonnet-4-20250514".to_string(),
                        directory: None,
                        worktree: Some("cli".to_string()),
                        env: None,
                        mcp_servers: None,
                        instructions: None,
                        startup_prompt: None,
                        capabilities: None,
                        nudge: None,
                    },
                    AgentConfig {
                        id: "testing".to_string(),
                        role: "QA Engineer".to_string(),
                        focus: "Unit tests, integration tests, and quality assurance".to_string(),
                        model: "claude-sonnet-4-20250514".to_string(),
                        directory: None,
                        worktree: Some("testing".to_string()),
                        env: None,
                        mcp_servers: None,
                        instructions: None,
                        startup_prompt: None,
                        capabilities: None,
                        nudge: None,
                    },
                ],
                McpRegistry::for_executor(),
            )
        }
        2 => {
            // Data Analysis
            (
                vec![
                    AgentConfig {
                        id: "analyst".to_string(),
                        role: "Data Analyst".to_string(),
                        focus: "Data exploration, analysis, and insights generation".to_string(),
                        model: "claude-opus-4-20250514".to_string(), // Use Opus for complex analysis
                        directory: None,
                        worktree: Some("analysis".to_string()),
                        env: None,
                        mcp_servers: None,
                        instructions: None,
                        startup_prompt: None,
                        capabilities: None,
                        nudge: None,
                    },
                    AgentConfig {
                        id: "visualization".to_string(),
                        role: "Visualization Engineer".to_string(),
                        focus: "Data visualization, charts, and reporting".to_string(),
                        model: "claude-sonnet-4-20250514".to_string(),
                        directory: None,
                        worktree: Some("viz".to_string()),
                        env: None,
                        mcp_servers: None,
                        instructions: None,
                        startup_prompt: None,
                        capabilities: None,
                        nudge: None,
                    },
                ],
                McpRegistry::for_data_analysis(),
            )
        }
        3 => {
            // Automation
            (
                vec![
                    AgentConfig {
                        id: "automation".to_string(),
                        role: "Automation Engineer".to_string(),
                        focus: "Scripts, bots, and automated workflows".to_string(),
                        model: "claude-sonnet-4-20250514".to_string(),
                        directory: None,
                        worktree: Some("automation".to_string()),
                        env: None,
                        mcp_servers: None,
                        instructions: None,
                        startup_prompt: None,
                        capabilities: None,
                        nudge: None,
                    },
                ],
                McpRegistry::for_automation(),
            )
        }
        _ => {
            // General Development
            (
                vec![
                    AgentConfig {
                        id: "dev-1".to_string(),
                        role: "Software Developer".to_string(),
                        focus: "General software development and implementation".to_string(),
                        model: "claude-sonnet-4-20250514".to_string(),
                        directory: None,
                        worktree: Some("dev1".to_string()),
                        env: None,
                        mcp_servers: None,
                        instructions: None,
                        startup_prompt: None,
                        capabilities: None,
                        nudge: None,
                    },
                    AgentConfig {
                        id: "dev-2".to_string(),
                        role: "Software Developer".to_string(),
                        focus: "General software development and implementation".to_string(),
                        model: "claude-sonnet-4-20250514".to_string(),
                        directory: None,
                        worktree: Some("dev2".to_string()),
                        env: None,
                        mcp_servers: None,
                        instructions: None,
                        startup_prompt: None,
                        capabilities: None,
                        nudge: None,
                    },
                ],
                McpRegistry::for_executor(),
            )
        }
    };

    // Ask about executor
    println!();
    let enable_executor = utils::confirm("Enable MCP Executor for running scripts and workflows?");

    let executor = if enable_executor {
        // Show recommended MCP servers
        println!();
        println!("{}", "Recommended MCP Servers for Executor:".bold());
        for (i, server) in default_mcp_servers.iter().enumerate() {
            println!("  {}. {} - {}", i + 1, server.name.cyan(), server.description);
        }
        println!();

        let use_recommended = utils::confirm("Use these recommended MCP servers?");

        let mcp_servers = if use_recommended {
            let mut servers = HashMap::new();
            let server_ids: Vec<String> = default_mcp_servers.iter().map(|s| s.id.clone()).collect();

            // Check for overlaps and show warnings
            let warnings = McpRegistry::detect_overlaps(&server_ids);
            if !warnings.is_empty() {
                println!();
                println!("{}", "Overlap Analysis:".yellow());
                for warning in warnings {
                    println!("  {}", warning);
                }
            }

            // Show complementary suggestions
            let suggestions = McpRegistry::suggest_complementary(&server_ids);
            if !suggestions.is_empty() {
                println!();
                println!("{}", "Suggested additions:".cyan());
                for (id, reason) in suggestions {
                    println!("  • {} - {}", id.cyan(), reason);
                }
            }

            for server in default_mcp_servers {
                servers.insert(server.id.clone(), server.config);
            }
            Some(servers)
        } else {
            None
        };

        Some(ExecutorConfig {
            enabled: true,
            agent_id: "mcp-executor".to_string(),
            mcp_servers,
            languages: vec!["typescript".to_string(), "python".to_string()],
        })
    } else {
        None
    };

    // Quick telemetry opt-in
    println!();
    let enable_telemetry = utils::confirm("Enable anonymous telemetry to help improve Colony?");
    let mut telemetry = crate::colony::config::TelemetryConfig::default();
    if enable_telemetry {
        telemetry.enabled = true;
        telemetry.get_or_create_anonymous_id();
    }

    println!();
    utils::success("Configuration created with smart defaults!");
    utils::info("You can customize this later by editing colony.yml or using 'colony tui'");

    Ok(ColonyConfig {
        name: None,
        repository: None,
        agents,
        executor,
        shared_state: None,
        auth: Default::default(),
        telemetry,
        capabilities: None,
        layout: None,
    })
}
