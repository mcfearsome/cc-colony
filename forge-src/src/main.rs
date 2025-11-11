use clap::{Parser, Subcommand};
use colored::Colorize;

mod api;
mod bundle;
mod config;
mod error;
mod swarm;
mod utils;

use error::ForgeResult;

#[derive(Parser)]
#[command(name = "forge")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Set up forge in current directory
    Init,

    /// Authenticate with useforge.cc API
    Login,

    /// Show installed MCP servers
    List,

    /// Search available MCP servers
    Search {
        /// Search query
        query: Option<String>,
    },

    /// Install an MCP server
    Add {
        /// Name of the server to install
        server_name: String,
    },

    /// Uninstall an MCP server
    Remove {
        /// Name of the server to remove
        server_name: String,
    },

    /// Bundle management commands
    Bundle {
        #[command(subcommand)]
        command: BundleCommands,
    },

    /// Multi-agent orchestration for Claude Code
    Swarm {
        #[command(subcommand)]
        command: SwarmCommands,
    },

    /// Pull latest recommendations from cloud
    Sync,

    /// Check for conflicts and issues
    Doctor,

    /// Export configuration backup
    Export {
        /// Output file path
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Import configuration from backup
    Import {
        /// Input file path
        file: String,
    },
}

#[derive(Subcommand)]
enum BundleCommands {
    /// Show available bundles
    List,

    /// Apply a bundle configuration
    Activate {
        /// Name of the bundle to activate
        name: String,
    },
}

#[derive(Subcommand)]
enum SwarmCommands {
    /// Initialize a new swarm configuration
    Init,

    /// Start all agents in the swarm (requires tmux)
    Start {
        /// Don't automatically attach to the tmux session after starting
        #[arg(long)]
        no_attach: bool,
    },

    /// Attach to the tmux session to watch agents work
    Attach,

    /// Interactive TUI for monitoring and controlling the swarm
    Tui,

    /// Show status of running agents
    Status,

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
    },

    /// Destroy the swarm and clean up resources
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

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

async fn run() -> ForgeResult<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => bundle::init::run().await,
        Commands::Login => bundle::login::run().await,
        Commands::List => bundle::list::run().await,
        Commands::Search { query } => bundle::search::run(query).await,
        Commands::Add { server_name } => bundle::add::run(server_name).await,
        Commands::Remove { server_name } => bundle::remove::run(server_name).await,
        Commands::Bundle { command } => match command {
            BundleCommands::List => bundle::bundle::list().await,
            BundleCommands::Activate { name } => bundle::bundle::activate(name).await,
        },
        Commands::Swarm { command } => match command {
            SwarmCommands::Init => swarm::init::run().await,
            SwarmCommands::Start { no_attach } => swarm::start::run(no_attach).await,
            SwarmCommands::Attach => swarm::attach::run().await,
            SwarmCommands::Tui => {
                let config_path = std::path::Path::new("swarm.yml");
                swarm::tui::run_tui(config_path)
                    .map_err(|e| crate::error::ForgeError::Swarm(e))?;
                Ok(())
            }
            SwarmCommands::Status => swarm::status::run().await,
            SwarmCommands::Broadcast { message } => swarm::broadcast::run(message).await,
            SwarmCommands::Stop { agent_id } => swarm::stop::run(agent_id).await,
            SwarmCommands::Logs { agent_id } => swarm::logs::run(agent_id).await,
            SwarmCommands::Destroy => swarm::destroy::run().await,
            SwarmCommands::Messages { command } => match command {
                MessageCommands::List { agent_id } => {
                    swarm::messages_cmd::list_messages(agent_id).await
                }
                MessageCommands::All => swarm::messages_cmd::list_all_messages().await,
            },
            SwarmCommands::Tasks { command } => match command {
                TaskCommands::List { status, compact } => {
                    swarm::tasks_cmd::list_tasks(status, compact).await
                }
                TaskCommands::Show { task_id } => swarm::tasks_cmd::show_task(task_id).await,
                TaskCommands::Create {
                    task_id,
                    title,
                    description,
                    assigned_to,
                    priority,
                } => {
                    swarm::tasks_cmd::create_task(
                        task_id,
                        title,
                        description,
                        assigned_to,
                        priority,
                    )
                    .await
                }
                TaskCommands::Claim { task_id, agent_id } => {
                    swarm::tasks_cmd::claim_task(task_id, agent_id).await
                }
                TaskCommands::Progress { task_id, progress } => {
                    swarm::tasks_cmd::update_task_progress(task_id, progress).await
                }
                TaskCommands::Block { task_id, reason } => {
                    swarm::tasks_cmd::block_task(task_id, reason).await
                }
                TaskCommands::Complete { task_id } => {
                    swarm::tasks_cmd::complete_task(task_id).await
                }
                TaskCommands::Unblock { task_id } => swarm::tasks_cmd::unblock_task(task_id).await,
                TaskCommands::Cancel { task_id } => swarm::tasks_cmd::cancel_task(task_id).await,
                TaskCommands::Delete { task_id } => swarm::tasks_cmd::delete_task(task_id).await,
                TaskCommands::Agent { agent_id } => {
                    swarm::tasks_cmd::list_tasks_for_agent(agent_id).await
                }
                TaskCommands::Claimable { agent_id } => {
                    swarm::tasks_cmd::list_claimable_tasks(agent_id).await
                }
            },
        },
        Commands::Sync => bundle::sync::run().await,
        Commands::Doctor => bundle::doctor::run().await,
        Commands::Export { output } => bundle::export::run(output).await,
        Commands::Import { file } => bundle::import::run(file).await,
    }
}
