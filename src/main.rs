use clap::{Parser, Subcommand};
use colored::Colorize;

mod colony;
mod error;
mod utils;

use error::ColonyResult;

#[derive(Parser)]
#[command(name = "colony")]
#[command(author, version, about = "Multi-agent orchestration for Claude Code on tmux", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new colony configuration
    Init,

    /// Start all agents in the colony (requires tmux)
    Start {
        /// Don't automatically attach to the tmux session after starting
        #[arg(long)]
        no_attach: bool,
    },

    /// Attach to the tmux session to watch agents work
    Attach,

    /// Manage authentication (API keys, OAuth, Bedrock)
    Auth {
        #[command(subcommand)]
        command: AuthCommands,
    },

    /// Connect to relay service for mobile/web control
    Relay {
        #[command(subcommand)]
        command: RelayCommands,
    },

    /// Interactive TUI for monitoring and controlling the colony
    Tui,

    /// Launch web dashboard in embedded webview (requires --features webview)
    #[cfg(feature = "webview")]
    Dashboard,

    /// Show status of running agents
    Status,

    /// Check colony system health
    Health,

    /// Broadcast a message to all agents
    Broadcast {
        /// Message to broadcast
        message: String,
    },

    /// Stop one or all agents
    Stop {
        /// Agent ID to stop (omit to stop all)
        agent_id: Option<String>,
    },

    /// View agent logs
    Logs {
        /// Agent ID to view logs for (omit to list all)
        agent_id: Option<String>,

        /// Filter by log level (debug, info, warn, error)
        #[arg(long)]
        level: Option<String>,

        /// Search for pattern in messages
        #[arg(long)]
        pattern: Option<String>,

        /// Show last N lines
        #[arg(short = 'n', long)]
        lines: Option<usize>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Disable colored output
        #[arg(long)]
        no_color: bool,
    },

    /// Destroy the colony and clean up resources
    Destroy,

    /// View and manage messages
    Messages {
        #[command(subcommand)]
        command: MessageCommands,
    },

    /// List and manage tasks
    Tasks {
        #[command(subcommand)]
        command: TaskCommands,
    },

    /// Manage shared state (tasks, workflows, memory)
    State {
        #[command(subcommand)]
        command: StateCommands,
    },

    /// Manage workflow orchestration
    Workflow {
        #[command(subcommand)]
        command: WorkflowOrchestratorCommands,
    },

    /// Manage plugins
    Plugin {
        #[command(subcommand)]
        command: PluginCommands,
    },

    /// Manage agent templates
    Template {
        #[command(subcommand)]
        command: TemplateCommands,
    },

    /// View and manage metrics
    Metrics {
        #[command(subcommand)]
        command: MetricsCommands,
    },
}

#[derive(Subcommand)]
enum MessageCommands {
    /// List messages for a specific agent
    List {
        /// Agent ID to view messages for
        agent_id: String,
    },

    /// List all messages in the system
    All,
}

#[derive(Subcommand)]
enum AuthCommands {
    /// Authenticate with OAuth (for Claude Pro/Max users)
    Login {
        /// Authentication method
        #[arg(long, value_enum, default_value = "oauth")]
        method: AuthMethod,

        /// API key (for api-key method)
        #[arg(long)]
        api_key: Option<String>,

        /// AWS region (for bedrock method)
        #[arg(long)]
        region: Option<String>,

        /// AWS profile (for bedrock method)
        #[arg(long)]
        profile: Option<String>,
    },

    /// Show authentication status
    Status,

    /// Logout (remove credentials)
    Logout,

    /// Refresh authentication token
    Refresh,
}

#[derive(Subcommand)]
enum RelayCommands {
    /// Connect to relay service
    Connect {
        /// Relay service URL (default: wss://api.colony.sh)
        #[arg(long)]
        url: Option<String>,

        /// Authentication token
        #[arg(long)]
        token: Option<String>,
    },

    /// Show relay connection status
    Status,

    /// Disconnect from relay service
    Disconnect,
}

#[derive(clap::ValueEnum, Clone)]
enum AuthMethod {
    /// OAuth with Claude.ai (Pro/Max users)
    OAuth,
    /// Anthropic API key
    ApiKey,
    /// AWS Bedrock
    Bedrock,
    /// Google Cloud Vertex AI
    VertexAi,
}

#[derive(Subcommand)]
enum TaskCommands {
    /// List all tasks
    List {
        /// Filter by status (pending, claimed, in_progress, blocked, completed)
        #[arg(short, long)]
        status: Option<String>,
        /// Use compact view
        #[arg(short, long)]
        compact: bool,
    },

    /// Show details for a specific task
    Show {
        /// Task ID
        task_id: String,
    },

    /// Create a new task
    Create {
        /// Task ID
        task_id: String,
        /// Task title
        title: String,
        /// Task description
        description: String,
        /// Assigned agent ID
        #[arg(short, long)]
        assigned_to: Option<String>,
        /// Priority (low, medium, high, critical)
        #[arg(short, long)]
        priority: Option<String>,
    },

    /// Claim a task for an agent
    Claim {
        /// Task ID
        task_id: String,
        /// Agent ID
        agent_id: String,
    },

    /// Update task progress
    Progress {
        /// Task ID
        task_id: String,
        /// Progress percentage (0-100)
        progress: u8,
    },

    /// Mark a task as blocked
    Block {
        /// Task ID
        task_id: String,
        /// Reason for blocking
        reason: String,
    },

    /// Complete a task
    Complete {
        /// Task ID
        task_id: String,
    },

    /// Unblock a task
    Unblock {
        /// Task ID
        task_id: String,
    },

    /// Cancel a task
    Cancel {
        /// Task ID
        task_id: String,
    },

    /// Delete a task
    Delete {
        /// Task ID
        task_id: String,
    },

    /// List tasks for a specific agent
    Agent {
        /// Agent ID
        agent_id: String,
    },

    /// List tasks an agent can claim
    Claimable {
        /// Agent ID
        agent_id: String,
    },
}

#[derive(Subcommand)]
enum StateCommands {
    /// Task management
    Task {
        #[command(subcommand)]
        command: TaskStateCommands,
    },

    /// Workflow management
    Workflow {
        #[command(subcommand)]
        command: WorkflowCommands,
    },

    /// Memory management
    Memory {
        #[command(subcommand)]
        command: MemoryCommands,
    },

    /// Pull latest state from remote
    Pull,

    /// Push local state to remote
    Push,

    /// Full sync (pull + push)
    Sync,
}

#[derive(Subcommand)]
enum TaskStateCommands {
    /// List all tasks
    List,

    /// List ready tasks (no blockers)
    Ready,

    /// Show task details
    Show {
        /// Task ID
        id: String,
    },

    /// Create a new task
    Add {
        /// Task title
        title: String,

        /// Task description
        #[arg(short, long)]
        description: Option<String>,

        /// Blocker task IDs (comma-separated)
        #[arg(short, long)]
        blockers: Option<String>,
    },

    /// Update task status
    Update {
        /// Task ID
        id: String,

        /// New status (ready, blocked, in_progress, completed, cancelled)
        status: String,
    },

    /// Assign task to agent
    Assign {
        /// Task ID
        id: String,

        /// Agent ID
        agent: String,
    },

    /// Add blocker to task
    Block {
        /// Task ID
        id: String,

        /// Blocker task ID
        blocker: String,
    },
}

#[derive(Subcommand)]
enum WorkflowCommands {
    /// List all workflows
    List,

    /// Show workflow details
    Show {
        /// Workflow ID
        id: String,
    },

    /// Create a new workflow
    Add {
        /// Workflow name
        name: String,
    },

    /// Update workflow status
    Update {
        /// Workflow ID
        id: String,

        /// New status (pending, running, completed, failed)
        status: String,
    },
}

#[derive(Subcommand)]
enum MemoryCommands {
    /// Add memory entry
    Add {
        /// Entry type (context, learned, decision, note)
        entry_type: String,

        /// Content
        content: String,

        /// Optional key (for context type)
        #[arg(short, long)]
        key: Option<String>,

        /// Optional value (for context type)
        #[arg(short, long)]
        value: Option<String>,
    },

    /// Search memory entries
    Search {
        /// Search query
        query: String,
    },
}

#[derive(Subcommand)]
enum WorkflowOrchestratorCommands {
    /// List all workflow definitions
    List,

    /// Show workflow definition details
    Show {
        /// Workflow name
        name: String,
    },

    /// Run a workflow
    Run {
        /// Workflow name
        name: String,

        /// Input JSON (optional)
        #[arg(short, long)]
        input: Option<String>,
    },

    /// Show workflow run status
    Status {
        /// Run ID
        run_id: String,
    },

    /// Show workflow run history
    History {
        /// Workflow name
        name: String,

        /// Limit number of results
        #[arg(short, long)]
        limit: Option<usize>,
    },

    /// Cancel a workflow run
    Cancel {
        /// Run ID
        run_id: String,
    },
}

#[derive(Subcommand)]
enum PluginCommands {
    /// List all installed plugins
    List,

    /// Show plugin details
    Show {
        /// Plugin name
        name: String,
    },

    /// Enable a plugin
    Enable {
        /// Plugin name
        name: String,
    },

    /// Disable a plugin
    Disable {
        /// Plugin name
        name: String,
    },
}

#[derive(Subcommand)]
enum TemplateCommands {
    /// List all available templates
    List,

    /// Show template details
    Show {
        /// Template name
        name: String,
    },

    /// Install a built-in template
    Install {
        /// Template name
        name: String,
    },

    /// List built-in templates
    Builtin,
}

#[derive(Subcommand)]
enum MetricsCommands {
    /// List all registered metrics
    List,

    /// Show detailed statistics for a metric
    Show {
        /// Metric name
        name: String,

        /// Time period in hours (default: 1)
        #[arg(long, default_value = "1")]
        hours: usize,
    },

    /// Export metrics to JSON
    Export {
        /// Output file path (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Clear old metrics data
    Clear {
        /// Clear all metrics (not just old data)
        #[arg(long)]
        all: bool,
    },

    /// Initialize sample metrics (for testing)
    Init,

    /// Record a sample metric value (for testing)
    Record {
        /// Metric name
        name: String,
        /// Value to record
        value: f64,
    },
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

async fn run() -> ColonyResult<()> {
    let cli = Cli::parse();

    // Initialize telemetry if config exists
    let telemetry_client = init_telemetry().await;

    // Get command name for telemetry
    let command_name = get_command_name(&cli.command);

    // Track command execution
    let start_time = std::time::Instant::now();

    let result = match cli.command {
        Commands::Init => colony::init::run().await,
        Commands::Start { no_attach } => colony::start::run(no_attach).await,
        Commands::Attach => colony::attach::run().await,
        Commands::Auth { command } => match command {
            AuthCommands::Login {
                method,
                api_key,
                region,
                profile,
            } => match method {
                AuthMethod::OAuth => colony::auth_cmd::login_oauth().await,
                AuthMethod::ApiKey => colony::auth_cmd::login_api_key(api_key).await,
                AuthMethod::Bedrock => colony::auth_cmd::login_bedrock(region, profile).await,
                AuthMethod::VertexAi => {
                    println!("Vertex AI authentication not yet implemented");
                    Ok(())
                }
            },
            AuthCommands::Status => colony::auth_cmd::status().await,
            AuthCommands::Logout => colony::auth_cmd::logout().await,
            AuthCommands::Refresh => colony::auth_cmd::refresh().await,
        },
        Commands::Relay { command } => match command {
            RelayCommands::Connect { url, token } => colony::relay_cmd::connect(url, token).await,
            RelayCommands::Status => colony::relay_cmd::status().await,
            RelayCommands::Disconnect => colony::relay_cmd::disconnect().await,
        },
        Commands::Tui => {
            let config_path = std::path::Path::new("colony.yml");
            colony::tui::run_tui(config_path).map_err(crate::error::ColonyError::Colony)?;
            Ok(())
        }
        #[cfg(feature = "webview")]
        Commands::Dashboard => {
            colony::ui::show_dashboard().map_err(|e| {
                crate::error::ColonyError::Colony(format!("Failed to launch dashboard: {}", e))
            })?;
            Ok(())
        }
        Commands::Status => colony::status::run().await,
        Commands::Health => colony::health::run().await,
        Commands::Broadcast { message } => colony::broadcast::run(message).await,
        Commands::Stop { agent_id } => colony::stop::run(agent_id).await,
        Commands::Logs {
            agent_id,
            level,
            pattern,
            lines,
            json,
            no_color,
        } => {
            colony::logs::run_with_options(
                agent_id,
                level.as_deref(),
                pattern.as_deref(),
                lines,
                json,
                no_color,
            )
            .await
        }
        Commands::Destroy => colony::destroy::run().await,
        Commands::Messages { command } => match command {
            MessageCommands::List { agent_id } => {
                colony::messages_cmd::list_messages(agent_id).await
            }
            MessageCommands::All => colony::messages_cmd::list_all_messages().await,
        },
        Commands::Tasks { command } => match command {
            TaskCommands::List { status, compact } => {
                colony::tasks_cmd::list_tasks(status, compact).await
            }
            TaskCommands::Show { task_id } => colony::tasks_cmd::show_task(task_id).await,
            TaskCommands::Create {
                task_id,
                title,
                description,
                assigned_to,
                priority,
            } => {
                colony::tasks_cmd::create_task(task_id, title, description, assigned_to, priority)
                    .await
            }
            TaskCommands::Claim { task_id, agent_id } => {
                colony::tasks_cmd::claim_task(task_id, agent_id).await
            }
            TaskCommands::Progress { task_id, progress } => {
                colony::tasks_cmd::update_task_progress(task_id, progress).await
            }
            TaskCommands::Block { task_id, reason } => {
                colony::tasks_cmd::block_task(task_id, reason).await
            }
            TaskCommands::Complete { task_id } => colony::tasks_cmd::complete_task(task_id).await,
            TaskCommands::Unblock { task_id } => colony::tasks_cmd::unblock_task(task_id).await,
            TaskCommands::Cancel { task_id } => colony::tasks_cmd::cancel_task(task_id).await,
            TaskCommands::Delete { task_id } => colony::tasks_cmd::delete_task(task_id).await,
            TaskCommands::Agent { agent_id } => {
                colony::tasks_cmd::list_tasks_for_agent(agent_id).await
            }
            TaskCommands::Claimable { agent_id } => {
                colony::tasks_cmd::list_claimable_tasks(agent_id).await
            }
        },
        Commands::State { command } => match command {
            StateCommands::Task { command } => match command {
                TaskStateCommands::List => colony::state_cmd::task_list().await,
                TaskStateCommands::Ready => colony::state_cmd::task_ready().await,
                TaskStateCommands::Show { id } => colony::state_cmd::task_show(id).await,
                TaskStateCommands::Add {
                    title,
                    description,
                    blockers,
                } => {
                    let blocker_vec = blockers
                        .map(|b| b.split(',').map(|s| s.trim().to_string()).collect())
                        .unwrap_or_else(Vec::new);
                    colony::state_cmd::task_add(title, description, blocker_vec).await
                }
                TaskStateCommands::Update { id, status } => {
                    colony::state_cmd::task_update(id, status).await
                }
                TaskStateCommands::Assign { id, agent } => {
                    colony::state_cmd::task_assign(id, agent).await
                }
                TaskStateCommands::Block { id, blocker } => {
                    colony::state_cmd::task_block(id, blocker).await
                }
            },
            StateCommands::Workflow { command } => match command {
                WorkflowCommands::List => colony::state_cmd::workflow_list().await,
                WorkflowCommands::Show { id } => colony::state_cmd::workflow_show(id).await,
                WorkflowCommands::Add { name } => colony::state_cmd::workflow_add(name).await,
                WorkflowCommands::Update { id, status } => {
                    colony::state_cmd::workflow_update(id, status).await
                }
            },
            StateCommands::Memory { command } => match command {
                MemoryCommands::Add {
                    entry_type,
                    content,
                    key,
                    value,
                } => colony::state_cmd::memory_add(entry_type, content, key, value).await,
                MemoryCommands::Search { query } => colony::state_cmd::memory_search(query).await,
            },
            StateCommands::Pull => colony::state_cmd::pull().await,
            StateCommands::Push => colony::state_cmd::push().await,
            StateCommands::Sync => colony::state_cmd::sync().await,
        },
        Commands::Workflow { command } => match command {
            WorkflowOrchestratorCommands::List => colony::workflow_cmd::list_workflows(),
            WorkflowOrchestratorCommands::Show { name } => {
                colony::workflow_cmd::show_workflow(&name)
            }
            WorkflowOrchestratorCommands::Run { name, input } => {
                colony::workflow_cmd::run_workflow(&name, input.as_deref())
            }
            WorkflowOrchestratorCommands::Status { run_id } => {
                colony::workflow_cmd::show_run_status(&run_id)
            }
            WorkflowOrchestratorCommands::History { name, limit } => {
                colony::workflow_cmd::list_run_history(&name, limit)
            }
            WorkflowOrchestratorCommands::Cancel { run_id } => {
                colony::workflow_cmd::cancel_run(&run_id)
            }
        },
        Commands::Plugin { command } => match command {
            PluginCommands::List => colony::plugin_cmd::list_plugins(),
            PluginCommands::Show { name } => colony::plugin_cmd::show_plugin(&name),
            PluginCommands::Enable { name } => colony::plugin_cmd::enable_plugin(&name),
            PluginCommands::Disable { name } => colony::plugin_cmd::disable_plugin(&name),
        },
        Commands::Template { command } => match command {
            TemplateCommands::List => colony::template_cmd::list_templates(),
            TemplateCommands::Show { name } => colony::template_cmd::show_template(&name),
            TemplateCommands::Install { name } => colony::template_cmd::install_template(&name),
            TemplateCommands::Builtin => colony::template_cmd::list_builtin(),
        },
        Commands::Metrics { command } => match command {
            MetricsCommands::List => colony::metrics_cmd::list_metrics(),
            MetricsCommands::Show { name, hours } => {
                colony::metrics_cmd::show_metric(&name, Some(hours))
            }
            MetricsCommands::Export { output } => {
                colony::metrics_cmd::export_metrics(output.as_deref())
            }
            MetricsCommands::Clear { all } => colony::metrics_cmd::clear_metrics(all),
            MetricsCommands::Init => colony::metrics_cmd::init_sample_metrics(),
            MetricsCommands::Record { name, value } => {
                colony::metrics_cmd::record_sample(&name, value)
            }
        },
    };

    // Track command completion/error
    let duration_ms = start_time.elapsed().as_millis() as u64;

    if let Some(client) = telemetry_client {
        match &result {
            Ok(_) => {
                client.track_command(&command_name, Some(duration_ms)).await;
            }
            Err(e) => {
                let error_type = format!("{:?}", e);
                client.track_error(&error_type, &command_name).await;
                client.track_command(&command_name, Some(duration_ms)).await;
            }
        }
    }

    result
}

/// Initialize telemetry client if config exists and telemetry is enabled
async fn init_telemetry() -> Option<colony::telemetry::TelemetryClient> {
    let config_path = std::path::Path::new("colony.yml");

    if !config_path.exists() {
        return None;
    }

    match colony::config::ColonyConfig::load(config_path) {
        Ok(config) => {
            if config.telemetry.enabled {
                Some(colony::telemetry::TelemetryClient::new(config.telemetry))
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

/// Get a human-readable command name for telemetry
fn get_command_name(command: &Commands) -> String {
    match command {
        Commands::Init => "init".to_string(),
        Commands::Start { .. } => "start".to_string(),
        Commands::Attach => "attach".to_string(),
        Commands::Auth { .. } => "auth".to_string(),
        Commands::Relay { .. } => "relay".to_string(),
        Commands::Tui => "tui".to_string(),
        #[cfg(feature = "webview")]
        Commands::Dashboard => "dashboard".to_string(),
        Commands::Status => "status".to_string(),
        Commands::Health => "health".to_string(),
        Commands::Broadcast { .. } => "broadcast".to_string(),
        Commands::Stop { .. } => "stop".to_string(),
        Commands::Logs { .. } => "logs".to_string(),
        Commands::Destroy => "destroy".to_string(),
        Commands::Messages { .. } => "messages".to_string(),
        Commands::Tasks { .. } => "tasks".to_string(),
        Commands::State { .. } => "state".to_string(),
        Commands::Workflow { .. } => "workflow".to_string(),
        Commands::Plugin { .. } => "plugin".to_string(),
        Commands::Template { .. } => "template".to_string(),
        Commands::Metrics { .. } => "metrics".to_string(),
    }
}
