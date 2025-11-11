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

    /// Interactive TUI for monitoring and controlling the colony
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

async fn run() -> ColonyResult<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => colony::init::run().await,
        Commands::Start { no_attach } => colony::start::run(no_attach).await,
        Commands::Attach => colony::attach::run().await,
        Commands::Tui => {
            let config_path = std::path::Path::new("colony.yml");
            colony::tui::run_tui(config_path)
                .map_err(|e| crate::error::ColonyError::Colony(e))?;
            Ok(())
        }
        Commands::Status => colony::status::run().await,
        Commands::Broadcast { message } => colony::broadcast::run(message).await,
        Commands::Stop { agent_id } => colony::stop::run(agent_id).await,
        Commands::Logs { agent_id } => colony::logs::run(agent_id).await,
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
                colony::tasks_cmd::create_task(
                    task_id,
                    title,
                    description,
                    assigned_to,
                    priority,
                )
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
            TaskCommands::Complete { task_id } => {
                colony::tasks_cmd::complete_task(task_id).await
            }
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
    }
}
