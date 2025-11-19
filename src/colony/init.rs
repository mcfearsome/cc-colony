use std::path::Path;

use crate::colony::config::{AgentConfig, ColonyConfig};
use crate::colony::controller::ColonyController;
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
    })
}
