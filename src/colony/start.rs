use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::colony::{executor, messaging, skills, state_integration, tmux, AgentStatus, ColonyConfig, ColonyController};
use crate::error::ColonyResult;
use crate::utils;

/// Start all agents in the colony
pub async fn run(no_attach: bool) -> ColonyResult<()> {
    let config_path = Path::new("colony.yml");

    if !config_path.exists() {
        return Err(crate::error::ColonyError::Colony(
            "colony.yml not found. Run 'colony init' first.".to_string(),
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

    utils::header("Starting Colony");
    utils::info(&format!("Session name: {}", session_name));

    // Install colony-message skill to ~/.claude/skills/ (system-wide)
    if let Err(e) = skills::install_colony_message_skill() {
        utils::warning(&format!(
            "Failed to install colony-message skill: {}. Agents may not have skill documentation.",
            e
        ));
    }

    // Create controller
    let mut controller = ColonyController::new(config)?;
    controller.initialize_agents()?;

    // Try to load previous state
    let _ = controller.load_state();

    // Initialize shared state system (if configured)
    if controller.config().shared_state.is_some() {
        let spinner = utils::spinner("Initializing shared state system...");
        setup_state_infrastructure(&controller).await?;
        spinner.finish_and_clear();
        utils::success("Shared state system ready");
    }

    // Create worktrees
    let spinner = utils::spinner("Creating Git worktrees...");
    controller.create_worktrees()?;
    spinner.finish_and_clear();
    utils::success("Created Git worktrees");

    // Set up executor environment BEFORE messaging (so executor project directory exists)
    if let Some(executor_config) = &controller.config().executor {
        if executor_config.enabled {
            utils::info("Setting up MCP Executor environment...");
            executor::setup_executor_environment(
                controller.colony_root(),
                executor_config,
            )?;
            utils::success("MCP Executor environment ready");
        }
    }

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

    // Clone repository config to avoid borrow checker issues in the loop
    let repo_config = controller.config().repository.clone();
    let has_shared_state = controller.config().shared_state.is_some();

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

        // Create startup prompt file and capture the prompt text
        let startup_prompt = match create_startup_prompt(agent, repo_config.as_ref(), has_shared_state).await {
            Ok(prompt) => Some(prompt),
            Err(e) => {
                utils::warning(&format!("  Failed to create startup prompt: {}", e));
                None
            }
        };

        // Create settings.json file if agent has MCP server configuration
        if agent.config.has_mcp_servers() {
            match create_agent_settings(agent).await {
                Ok(()) => {
                    utils::info(&format!("  Created settings.json with MCP server configuration"));
                }
                Err(e) => {
                    utils::warning(&format!("  Failed to create settings.json: {}", e));
                }
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

        // Build environment variable exports if configured
        let env_prefix = if let Some(env_vars) = &agent.config.env {
            let exports: Vec<String> = env_vars
                .iter()
                .map(|(key, value)| {
                    format!("export {}={}", shell_escape(key), shell_escape(value))
                })
                .collect();
            if exports.is_empty() {
                String::new()
            } else {
                format!("{} && ", exports.join(" && "))
            }
        } else {
            String::new()
        };

        // Build Claude command with optional settings path
        let claude_cmd = if agent.config.has_mcp_servers() {
            let settings_path = agent.project_path.join(".claude").join("settings.json");
            let settings_path_str = settings_path.to_str().ok_or_else(|| {
                crate::error::ColonyError::Colony(format!(
                    "Invalid settings path for agent '{}': contains non-UTF-8 characters",
                    agent.id()
                ))
            })?;
            format!(
                "{}cd {} && claude --project {} --settings {} --dangerously-skip-permissions",
                env_prefix,
                shell_escape(worktree_path_str),
                shell_escape(project_path_str),
                shell_escape(settings_path_str)
            )
        } else {
            format!(
                "{}cd {} && claude --project {} --dangerously-skip-permissions",
                env_prefix,
                shell_escape(worktree_path_str),
                shell_escape(project_path_str)
            )
        };

        // Track the actual pane index (tmux assigns these, not us)
        let pane_idx = if index > 0 {
            // Alternate between vertical and horizontal splits for better layout
            // Odd indices (1, 3, 5...) use vertical splits
            // Even indices (2, 4, 6...) use horizontal splits
            // This creates a more balanced grid-like layout
            const VERTICAL_SPLIT_MODULO: usize = 1;
            const HORIZONTAL_SPLIT_MODULO: usize = 0;

            if index % 2 == VERTICAL_SPLIT_MODULO {
                tmux::split_vertical(&session_name, &claude_cmd)?
            } else {
                tmux::split_horizontal(&session_name, &claude_cmd)?
            }
        } else {
            // For the first agent (index 0), send the command to the initial pane
            const FIRST_PANE_INDEX: usize = 0;
            tmux::send_command_to_pane(&session_name, FIRST_PANE_INDEX, &claude_cmd)?;
            FIRST_PANE_INDEX
        };

        // Set pane title using the actual pane index from tmux
        tmux::set_pane_title(&session_name, pane_idx, &format!("Agent: {}", agent.id()))?;

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
                "tmux pipe-pane -t {}:0.{} 'cat >> {}'",
                escaped_session, pane_idx, log_path
            );
            let _ = std::process::Command::new("sh")
                .arg("-c")
                .arg(&pipe_cmd)
                .output();
        }

        // Send startup prompt to the pane if available
        if let Some(prompt) = startup_prompt {
            // Wait a moment for Claude to initialize
            tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

            // Send the prompt via tmux send-keys using actual pane index
            if let Err(e) = send_prompt_to_pane(&session_name, pane_idx, &prompt) {
                utils::warning(&format!(
                    "  Failed to send startup prompt to agent '{}': {}",
                    agent.id(),
                    e
                ));
            } else {
                utils::info(&format!("  Sent startup prompt to agent '{}'", agent.id()));
            }
        }

        agent.set_status(AgentStatus::Running);
        utils::success(&format!(
            "  Started agent '{}' in pane {}",
            agent.id(),
            pane_idx
        ));

        println!();
    }

    // Create MCP Executor pane if enabled
    let mut executor_pane_index: Option<usize> = None;
    if let Some(executor_config) = &controller.config().executor {
        if executor_config.enabled {
            utils::info("Starting MCP Executor pane...");

            // Get the executor project path (environment already set up earlier)
            let executor_project_path = controller
                .colony_root()
                .join("projects")
                .join(&executor_config.agent_id);

            let executor_project_str = executor_project_path.to_str().ok_or_else(|| {
                crate::error::ColonyError::Colony(
                    "Invalid executor project path: contains non-UTF-8 characters".to_string(),
                )
            })?;

            // Get the current directory for the executor to run in
            let executor_work_dir = std::env::current_dir().map_err(|e| {
                crate::error::ColonyError::Colony(format!("Failed to get current directory: {}", e))
            })?;
            let executor_work_dir_str = executor_work_dir.to_str().ok_or_else(|| {
                crate::error::ColonyError::Colony(
                    "Current directory path contains non-UTF-8 characters".to_string(),
                )
            })?;

            // Build environment variables for the executor
            let executor_env = format!(
                "export COLONY_AGENT_ID={}",
                shell_escape(&executor_config.agent_id)
            );

            // Create settings.json for executor if it has MCP servers configured
            if executor_config.has_mcp_servers() {
                match create_executor_settings(&executor_project_path, executor_config).await {
                    Ok(()) => {
                        utils::info("  Created executor settings.json with MCP server configuration");
                    }
                    Err(e) => {
                        utils::warning(&format!(
                            "  Failed to create executor settings: {}. Executor will not have MCP servers.",
                            e
                        ));
                    }
                }
            }

            // Build the Claude command for the executor
            let executor_cmd = if executor_config.has_mcp_servers() {
                let executor_settings_path = executor_project_path.join(".claude/settings.json");
                let executor_settings_str = executor_settings_path.to_str().ok_or_else(|| {
                    crate::error::ColonyError::Colony(
                        "Invalid executor settings path: contains non-UTF-8 characters".to_string(),
                    )
                })?;
                format!(
                    "{} && cd {} && claude --project {} --settings {} --dangerously-skip-permissions",
                    executor_env,
                    shell_escape(executor_work_dir_str),
                    shell_escape(executor_project_str),
                    shell_escape(executor_settings_str)
                )
            } else {
                format!(
                    "{} && cd {} && claude --project {} --dangerously-skip-permissions",
                    executor_env,
                    shell_escape(executor_work_dir_str),
                    shell_escape(executor_project_str)
                )
            };

            // Create the executor pane and get actual pane index from tmux
            let executor_pane_idx = if agent_count > 0 {
                tmux::split_vertical(&session_name, &executor_cmd)?
            } else {
                // If no agents (unusual case), send to first pane
                tmux::send_command_to_pane(&session_name, 0, &executor_cmd)?;
                0
            };
            executor_pane_index = Some(executor_pane_idx);

            tmux::set_pane_title(
                &session_name,
                executor_pane_idx,
                &format!("MCP Executor: {}", executor_config.agent_id),
            )?;

            // Enable output capture for executor pane
            #[cfg(unix)]
            {
                let executor_log_path = controller
                    .colony_root()
                    .join("logs")
                    .join(format!("{}.log", executor_config.agent_id));
                let log_path_str = executor_log_path.to_str().ok_or_else(|| {
                    crate::error::ColonyError::Colony(
                        "Invalid executor log path: contains non-UTF-8 characters".to_string(),
                    )
                })?;
                let log_path = shell_escape(log_path_str);
                let escaped_session = shell_escape(&session_name);
                let pipe_cmd = format!(
                    "tmux pipe-pane -t {}:0.{} 'cat >> {}'",
                    escaped_session, executor_pane_idx, log_path
                );
                let _ = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(&pipe_cmd)
                    .output();
            }

            // Send startup prompt to the executor pane
            let executor_prompt_path = executor_project_path.join(".claude/startup_prompt.txt");
            if executor_prompt_path.exists() {
                match tokio::fs::read_to_string(&executor_prompt_path).await {
                    Ok(prompt) => {
                        // Wait for Claude to initialize
                        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

                        if let Err(e) = send_prompt_to_pane(&session_name, executor_pane_idx, &prompt) {
                            utils::warning(&format!(
                                "  Failed to send startup prompt to executor: {}",
                                e
                            ));
                        } else {
                            utils::info("  Sent startup prompt to executor");
                        }
                    }
                    Err(e) => {
                        utils::warning(&format!(
                            "  Failed to read executor startup prompt: {}",
                            e
                        ));
                    }
                }
            }

            utils::success(&format!(
                "  MCP Executor '{}' started in pane {}",
                executor_config.agent_id, executor_pane_idx
            ));
            println!();
        }
    }

    // Create TUI pane for monitoring
    utils::info("Setting up orchestration TUI...");

    // Get the path to the current colony executable
    let colony_binary = std::env::current_exe().map_err(|e| {
        crate::error::ColonyError::Colony(format!("Failed to get colony binary path: {}", e))
    })?;
    let colony_path = colony_binary.to_str().ok_or_else(|| {
        crate::error::ColonyError::Colony(
            "Colony binary path contains non-UTF-8 characters".to_string(),
        )
    })?;

    // Get current directory for the TUI to run in
    let current_dir = std::env::current_dir().map_err(|e| {
        crate::error::ColonyError::Colony(format!("Failed to get current directory: {}", e))
    })?;
    let current_dir_str = current_dir.to_str().ok_or_else(|| {
        crate::error::ColonyError::Colony(
            "Current directory path contains non-UTF-8 characters".to_string(),
        )
    })?;

    let tui_cmd = format!(
        "cd {} && {} colony tui",
        shell_escape(current_dir_str),
        shell_escape(colony_path)
    );

    // Create a pane for the TUI and get actual pane index from tmux
    if agent_count > 0 {
        let tui_pane_index = tmux::split_vertical(&session_name, &tui_cmd)?;
        tmux::set_pane_title(&session_name, tui_pane_index, "Orchestration TUI")?;
        utils::success(&format!("  Orchestration TUI pane created in pane {}", tui_pane_index));
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
        println!("  • Run 'colony attach' to view agents and TUI in tmux");
        println!("  • The orchestration TUI is already running in a dedicated pane");
        println!("  • Use 'colony status' to check agent status from CLI");
        println!("  • Use 'colony logs <agent-id>' to view specific logs");
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

/// Create a startup prompt file for an agent and return the prompt text
async fn create_startup_prompt(
    agent: &crate::colony::Agent,
    repo_config: Option<&crate::colony::config::RepositoryConfig>,
    has_shared_state: bool,
) -> ColonyResult<String> {
    let prompt_path = agent.project_path.join("startup_prompt.txt");

    // If a custom startup prompt is provided, use it directly
    let prompt = if let Some(custom_prompt) = &agent.config.startup_prompt {
        custom_prompt.clone()
    } else {
        // Build repository context section if available
        let repo_context = if let Some(repo_cfg) = repo_config {
            let mut context = format!("\n## Repository Context\n\n");
            context.push_str(&format!("**Type**: {}\n", repo_cfg.repo_type.description()));

            if let Some(purpose) = &repo_cfg.purpose {
                context.push_str(&format!("**Purpose**: {}\n", purpose));
            }

            if let Some(ctx) = &repo_cfg.context {
                context.push_str(&format!("\n{}\n", ctx));
            }

            context.push_str("\n");
            context
        } else {
            String::new()
        };

        // Build shared state section if configured
        let state_section = if has_shared_state {
            r#"

## Shared State System

You have access to a git-backed shared state system for coordinating work:

### Task Management
```bash
# List ready tasks (no blockers)
./colony_state.sh task ready

# Create a new task
./colony_state.sh task add "Task description"

# Assign task to yourself
./colony_state.sh task assign task-abc123

# Update task status
./colony_state.sh task update task-abc123 in_progress

# Mark task as completed
./colony_state.sh task update task-abc123 completed
```

### Workflows
```bash
# List all workflows
./colony_state.sh workflow list

# Create a new workflow
./colony_state.sh workflow add "Multi-step process"
```

### Memory & Context
```bash
# Store learned information
./colony_state.sh memory add learned "Important discovery"

# Store contextual info
./colony_state.sh memory add context "API_URL=https://api.example.com"
```

### Sync State
```bash
# Pull latest state (before starting work)
./colony_state.sh pull

# Push your changes (after completing work)
./colony_state.sh push

# Full sync (pull + push)
./colony_state.sh sync
```

**Best Practices**:
- Check for ready tasks before creating new ones
- Assign tasks to yourself when starting work
- Update task status as you progress
- Store important learnings in memory
- Sync state regularly to coordinate with others

For complete documentation:
`.colony/STATE_README.md`

"#
        } else {
            ""
        };

        // Otherwise, generate the default colony prompt
        let mut prompt = format!(
            r#"# Welcome to Colony

You are **{}** working as part of a multi-agent colony.
{}
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
{}
## Coordination

Read the full communication guide at:
`.colony/COLONY_COMMUNICATION.md`

For detailed messaging guidance, see the Colony Message skill:
`.claude/skills/colony-message.md`
"#,
            agent.id(),
            repo_context,
            agent.config.role,
            agent.config.focus,
            state_section
        );

        // Append custom instructions if provided
        if let Some(instructions) = &agent.config.instructions {
            prompt.push_str("\n\n---\n\n## Additional Instructions\n\n");
            prompt.push_str(instructions);
            prompt.push_str("\n");
        }

        prompt.push_str("\nNow get started on your assigned work! Remember to check for messages from your teammates.\n");
        prompt
    };

    let mut file = File::create(&prompt_path).await?;
    file.write_all(prompt.as_bytes()).await?;
    file.flush().await?;

    Ok(prompt)
}

/// Create a settings.json file for an agent with MCP server configuration
async fn create_agent_settings(agent: &crate::colony::Agent) -> ColonyResult<()> {
    use serde_json::Value;

    // Create .claude directory in the project path
    let claude_dir = agent.project_path.join(".claude");
    tokio::fs::create_dir_all(&claude_dir).await?;

    // Check if there's an existing settings.json in the working directory
    let worktree_settings_path = agent.worktree_path.join(".claude").join("settings.json");
    let mut merged_settings: Value = serde_json::json!({});

    if worktree_settings_path.exists() {
        // Load existing settings from the working directory
        match tokio::fs::read_to_string(&worktree_settings_path).await {
            Ok(contents) => {
                match serde_json::from_str::<Value>(&contents) {
                    Ok(existing_settings) => {
                        utils::info(&format!(
                            "  Found existing .claude/settings.json in working directory, merging..."
                        ));
                        merged_settings = existing_settings;
                    }
                    Err(e) => {
                        utils::warning(&format!(
                            "  Failed to parse existing settings.json: {}",
                            e
                        ));
                    }
                }
            }
            Err(e) => {
                utils::warning(&format!("  Failed to read existing settings.json: {}", e));
            }
        }
    }

    // Generate settings JSON from agent config
    let agent_settings_str = agent.config.generate_settings_json()?;
    let agent_settings: Value = serde_json::from_str(&agent_settings_str)?;

    // Merge agent settings into the base settings (agent settings take precedence)
    if let Some(agent_obj) = agent_settings.as_object() {
        if let Some(merged_obj) = merged_settings.as_object_mut() {
            for (key, value) in agent_obj {
                // For mcpServers, merge at the server level
                if key == "mcpServers" {
                    if let Some(existing_mcp) = merged_obj.get_mut("mcpServers") {
                        if let (Some(existing_map), Some(new_map)) =
                            (existing_mcp.as_object_mut(), value.as_object())
                        {
                            // Merge MCP servers, agent config overrides
                            for (server_name, server_config) in new_map {
                                existing_map.insert(server_name.clone(), server_config.clone());
                            }
                            continue;
                        }
                    }
                }
                // For other keys, agent config completely overrides
                merged_obj.insert(key.clone(), value.clone());
            }
        }
    }

    // If merged_settings is still empty, use agent settings directly
    if merged_settings.as_object().map_or(true, |o| o.is_empty()) {
        merged_settings = agent_settings;
    }

    // Write merged settings.json file
    let settings_json = serde_json::to_string_pretty(&merged_settings)
        .map_err(|e| crate::error::ColonyError::Colony(format!("Failed to serialize merged settings: {}", e)))?;

    let settings_path = claude_dir.join("settings.json");
    let mut file = File::create(&settings_path).await?;
    file.write_all(settings_json.as_bytes()).await?;
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

    // Create inbox for executor if enabled
    if let Some(executor_config) = &controller.config().executor {
        if executor_config.enabled {
            let inbox_dir = messages_dir.join(&executor_config.agent_id);
            std::fs::create_dir_all(&inbox_dir)?;

            let sent_dir = inbox_dir.join("sent");
            std::fs::create_dir_all(&sent_dir)?;
        }
    }

    // Create messaging README
    messaging::create_messaging_readme(colony_root)?;

    // Create helper scripts for each agent
    for agent in controller.agents().values() {
        messaging::create_message_helper_script(colony_root, agent.id())?;
    }

    // Create helper script for executor if enabled
    if let Some(executor_config) = &controller.config().executor {
        if executor_config.enabled {
            messaging::create_message_helper_script(colony_root, &executor_config.agent_id)?;
        }
    }

    Ok(())
}

/// Set up shared state infrastructure
async fn setup_state_infrastructure(controller: &ColonyController) -> ColonyResult<()> {
    use crate::colony::state::{GitBackedState, SharedStateConfig};

    let colony_root = controller.colony_root();
    let config = controller.config();

    // Get shared state config (we know it exists from the caller check)
    let state_config = config.shared_state.as_ref().unwrap();

    // Initialize GitBackedState backend
    let repo_root = std::env::current_dir()?;
    let state_backend = GitBackedState::new(state_config.clone(), repo_root)?;

    // Pull latest state if configured
    if state_config.auto_pull {
        utils::info("Pulling latest state from remote...");
        if let Err(e) = state_backend.pull().await {
            utils::warning(&format!("Failed to pull state: {}. Continuing with local state.", e));
        } else {
            utils::success("State synced from remote");
        }
    }

    // Create state README
    state_integration::create_state_readme(colony_root)?;

    // Create helper scripts for each agent
    for agent in controller.agents().values() {
        state_integration::create_state_helper_script(colony_root, agent.id())?;
    }

    // Create helper script for executor if enabled
    if let Some(executor_config) = &config.executor {
        if executor_config.enabled {
            state_integration::create_state_helper_script(colony_root, &executor_config.agent_id)?;
        }
    }

    Ok(())
}

/// Escape a string for safe use in shell commands
/// This prevents shell injection by wrapping in single quotes and escaping any single quotes
fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// Send a prompt to a tmux pane using send-keys
/// The prompt is sent directly to Claude Code running in the pane
fn send_prompt_to_pane(session_name: &str, pane_index: usize, prompt: &str) -> ColonyResult<()> {
    use std::process::Command;

    // Escape the prompt for safe shell usage
    let escaped_prompt = shell_escape(prompt);
    let escaped_session = shell_escape(session_name);

    // Use tmux send-keys to send the prompt to the pane
    // The prompt is sent as literal text (not executed as a command)
    // Use window.pane format (default window is 0)
    let send_cmd = format!(
        "tmux send-keys -t {}:0.{} -l {}",
        escaped_session, pane_index, escaped_prompt
    );

    let output = Command::new("sh").arg("-c").arg(&send_cmd).output()?;

    if !output.status.success() {
        return Err(crate::error::ColonyError::Colony(format!(
            "Failed to send prompt to pane: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    // Send Enter key to submit the prompt
    let enter_cmd = format!("tmux send-keys -t {}:0.{} Enter", escaped_session, pane_index);

    let output = Command::new("sh").arg("-c").arg(&enter_cmd).output()?;

    if !output.status.success() {
        return Err(crate::error::ColonyError::Colony(format!(
            "Failed to send Enter key to pane: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Create executor settings.json with MCP server configuration
/// Uses the same approach as agent settings
async fn create_executor_settings(
    executor_project_path: &Path,
    executor_config: &crate::colony::config::ExecutorConfig,
) -> ColonyResult<()> {
    // Create the executor's .claude directory
    let claude_dir = executor_project_path.join(".claude");
    tokio::fs::create_dir_all(&claude_dir).await?;

    // Generate settings JSON from executor config (same as agents)
    let settings_json_str = executor_config.generate_settings_json()?;

    // Write settings.json file
    let settings_path = claude_dir.join("settings.json");
    let mut file = File::create(&settings_path).await?;
    file.write_all(settings_json_str.as_bytes()).await?;
    file.flush().await?;

    Ok(())
}
