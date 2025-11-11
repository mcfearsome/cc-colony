use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::error::ColonyResult;
use crate::colony::{messaging, tmux, AgentStatus, ColonyConfig, ColonyController};
use crate::utils;

/// Start all agents in the colony
pub async fn run(no_attach: bool) -> ColonyResult<()> {
    let config_path = Path::new("colony.yml");

    if !config_path.exists() {
        return Err(crate::error::ColonyError::Colony(
            "colony.yml not found. Run 'forge colony init' first.".to_string(),
        ));
    }

    // Check if tmux is available - try to install if not
    if !tmux::is_tmux_available() {
        utils::warning("tmux is not installed. Attempting to install...");
        if let Err(e) = tmux::try_install_tmux() {
            return Err(crate::error::ColonyError::Colony(format!(
                "Failed to install tmux: {}\n\
                 Please install manually:\n\
                 - macOS: brew install tmux\n\
                 - Ubuntu/Debian: sudo apt-get install tmux\n\
                 - Fedora/RHEL: sudo dnf install tmux",
                e
            )));
        }
        utils::success("tmux installed successfully!");
    }

    // Load configuration
    let config = ColonyConfig::load(config_path)?;
    config.validate()?;

    // Get the session name from config
    let session_name = config.session_name();

    utils::header("Starting Forge Colony");
    utils::info(&format!("Session name: {}", session_name));

    // Create controller
    let mut controller = ColonyController::new(config)?;
    controller.initialize_agents()?;

    // Try to load previous state
    let _ = controller.load_state();

    // Create worktrees
    let spinner = utils::spinner("Creating Git worktrees...");
    controller.create_worktrees()?;
    spinner.finish_and_clear();
    utils::success("Created Git worktrees");

    // Set up messaging infrastructure
    let spinner = utils::spinner("Setting up message queue system...");
    setup_messaging_infrastructure(&controller)?;
    spinner.finish_and_clear();
    utils::success("Message queue system ready");

    // Kill existing session if it exists
    if tmux::session_exists(&session_name) {
        utils::info("Found existing tmux session, cleaning up...");
        tmux::kill_session(&session_name)?;
    }

    // Create tmux session
    utils::info(&format!("Creating tmux session '{}'...", session_name));
    tmux::create_session(&session_name)?;

    // Start each agent in a tmux pane
    let agent_ids: Vec<String> = controller.agents().keys().cloned().collect();
    let agent_count = agent_ids.len();

    for (index, agent_id) in agent_ids.iter().enumerate() {
        let agent = controller
            .agents_mut()
            .get_mut(agent_id)
            .expect("Agent ID should always exist in agents_mut; collected from agents().keys()");

        // Skip if already running
        if agent.is_running() {
            utils::warning(&format!("Agent '{}' is already running", agent.id()));
            continue;
        }

        utils::info(&format!("Starting agent: {}", agent.id()));
        println!("  Role:  {}", agent.config.role);
        println!("  Focus: {}", agent.config.focus);
        println!("  Model: {}", agent.config.model);

        // Create startup prompt file
        match create_startup_prompt(agent).await {
            Ok(()) => {}
            Err(e) => {
                utils::warning(&format!("  Failed to create startup prompt: {}", e));
            }
        }

        // Build the claude command with properly escaped paths
        let worktree_path_str = agent.worktree_path.to_str().ok_or_else(|| {
            crate::error::ColonyError::Colony(format!(
                "Invalid worktree path for agent '{}': contains non-UTF-8 characters",
                agent.id()
            ))
        })?;
        let project_path_str = agent.project_path.to_str().ok_or_else(|| {
            crate::error::ColonyError::Colony(format!(
                "Invalid project path for agent '{}': contains non-UTF-8 characters",
                agent.id()
            ))
        })?;

        let claude_cmd = format!(
            "cd {} && claude --project {} --dangerously-skip-permissions",
            shell_escape(worktree_path_str),
            shell_escape(project_path_str)
        );

        // If this is not the first agent, split the window
        if index > 0 {
            // Alternate between vertical and horizontal splits for better layout
            // Odd indices (1, 3, 5...) use vertical splits
            // Even indices (2, 4, 6...) use horizontal splits
            // This creates a more balanced grid-like layout
            const VERTICAL_SPLIT_MODULO: usize = 1;
            const HORIZONTAL_SPLIT_MODULO: usize = 0;

            if index % 2 == VERTICAL_SPLIT_MODULO {
                tmux::split_vertical(&session_name, &claude_cmd)?;
            } else if index % 2 == HORIZONTAL_SPLIT_MODULO {
                tmux::split_horizontal(&session_name, &claude_cmd)?;
            }
        } else {
            // For the first agent (index 0), send the command to the initial pane
            const FIRST_PANE_INDEX: usize = 0;
            tmux::send_command_to_pane(&session_name, FIRST_PANE_INDEX, &claude_cmd)?;
        }

        // Set pane title
        tmux::set_pane_title(&session_name, index, &format!("Agent: {}", agent.id()))?;

        // Enable output capture for this pane (pipe to log file)
        #[cfg(unix)]
        {
            let log_path_str = agent.log_path.to_str().ok_or_else(|| {
                crate::error::ColonyError::Colony(format!(
                    "Invalid log path for agent '{}': contains non-UTF-8 characters",
                    agent.id()
                ))
            })?;
            let log_path = shell_escape(log_path_str);
            let escaped_session = shell_escape(&session_name);
            let pipe_cmd = format!(
                "tmux pipe-pane -t {}:{} 'cat >> {}'",
                escaped_session, index, log_path
            );
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg(&pipe_cmd)
                .output();
        }

        agent.set_status(AgentStatus::Running);
        utils::success(&format!(
            "  Started agent '{}' in pane {}",
            agent.id(),
            index
        ));

        println!();
    }

    // Create TUI pane for monitoring
    utils::info("Setting up orchestration TUI...");

    // Get the path to the current forge executable
    let forge_binary = std::env::current_exe()
        .map_err(|e| crate::error::ColonyError::Colony(format!("Failed to get forge binary path: {}", e)))?;
    let forge_path = forge_binary.to_str().ok_or_else(|| {
        crate::error::ColonyError::Colony("Forge binary path contains non-UTF-8 characters".to_string())
    })?;

    // Get current directory for the TUI to run in
    let current_dir = std::env::current_dir()
        .map_err(|e| crate::error::ColonyError::Colony(format!("Failed to get current directory: {}", e)))?;
    let current_dir_str = current_dir.to_str().ok_or_else(|| {
        crate::error::ColonyError::Colony("Current directory path contains non-UTF-8 characters".to_string())
    })?;

    let tui_cmd = format!(
        "cd {} && {} colony tui",
        shell_escape(current_dir_str),
        shell_escape(forge_path)
    );

    // Create a pane for the TUI
    if agent_count > 0 {
        tmux::split_vertical(&session_name, &tui_cmd)?;
        let tui_pane_index = agent_count; // Next pane after all agents
        tmux::set_pane_title(&session_name, tui_pane_index, "Orchestration TUI")?;
        utils::success("  Orchestration TUI pane created");
    }

    // Apply tiled layout for better view
    if agent_count > 0 {
        tmux::select_tiled_layout(&session_name)?;
    }

    // Save state
    controller.save_state()?;

    utils::header("Colony Started Successfully!");
    let running_count = controller
        .agents()
        .values()
        .filter(|a| a.is_running())
        .count();
    println!(
        "Running agents: {}/{}",
        running_count,
        controller.agents().len()
    );

    if no_attach {
        println!("\nNext steps:");
        println!("  • Run 'forge colony attach' to view agents and TUI in tmux");
        println!("  • The orchestration TUI is already running in a dedicated pane");
        println!("  • Use 'forge colony status' to check agent status from CLI");
        println!("  • Use 'forge colony logs <agent-id>' to view specific logs");
        println!("\nTip: Press Ctrl+B then D to detach from tmux without stopping agents");
    } else {
        println!("\nAttaching to tmux session...");
        println!("Tip: Press Ctrl+B then D to detach from tmux without stopping agents");
        println!();

        // Small delay to ensure all panes are ready
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Attach to the session
        tmux::attach_session(&session_name)?;
    }

    Ok(())
}

/// Create a startup prompt file for an agent
async fn create_startup_prompt(agent: &crate::colony::Agent) -> ColonyResult<()> {
    let prompt_path = agent.project_path.join("startup_prompt.txt");
    let prompt = format!(
        r#"# Welcome to Forge Colony

You are **{}** working as part of a multi-agent colony.

## Your Role
**Role**: {}
**Focus**: {}

## Communication System

You can communicate with other agents using the message queue system:

### Checking Messages
```bash
./colony_message.sh read
```

### Sending Messages
```bash
# Send to a specific agent
./colony_message.sh send <agent-id> "Your message here"

# Broadcast to all agents
./colony_message.sh send all "Important announcement"
```

### List Other Agents
```bash
./colony_message.sh list-agents
```

## Best Practices

1. **Check messages regularly** - Run `./colony_message.sh read` periodically
2. **Announce what you're working on** - Avoid duplicate work
3. **Ask for help when blocked** - Other agents can assist
4. **Share important findings** - Keep the team informed
5. **Coordinate on shared resources** - Communicate before modifying shared files

## Message Examples

When starting work:
```bash
./colony_message.sh send all "Starting work on [task description]"
```

When you need help:
```bash
./colony_message.sh send <agent-id> "Need assistance with X"
```

When you complete something:
```bash
./colony_message.sh send all "Completed [task description] - ready for integration"
```

## Coordination

Read the full communication guide at:
`.forge-colony/COLONY_COMMUNICATION.md`

Now get started on your assigned work! Remember to check for messages from your teammates.
"#,
        agent.id(),
        agent.config.role,
        agent.config.focus
    );

    let mut file = File::create(&prompt_path).await?;
    file.write_all(prompt.as_bytes()).await?;
    file.flush().await?;

    Ok(())
}

/// Set up messaging infrastructure for the colony
fn setup_messaging_infrastructure(controller: &ColonyController) -> ColonyResult<()> {
    let colony_root = controller.colony_root();

    // Create messaging directory structure
    let messages_dir = colony_root.join("messages");
    std::fs::create_dir_all(&messages_dir)?;

    // Create broadcast directory
    std::fs::create_dir_all(messages_dir.join("broadcast"))?;

    // Create inbox for each agent
    for agent in controller.agents().values() {
        let inbox_dir = messages_dir.join(agent.id());
        std::fs::create_dir_all(&inbox_dir)?;

        let sent_dir = inbox_dir.join("sent");
        std::fs::create_dir_all(&sent_dir)?;
    }

    // Create messaging README
    messaging::create_messaging_readme(colony_root)?;

    // Create helper scripts for each agent
    for agent in controller.agents().values() {
        messaging::create_message_helper_script(colony_root, agent.id())?;
    }

    Ok(())
}

/// Escape a string for safe use in shell commands
/// This prevents shell injection by wrapping in single quotes and escaping any single quotes
fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}
