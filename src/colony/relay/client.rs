use crate::colony::controller::ColonyController;
use crate::error::{ColonyError, ColonyResult};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{interval, sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use super::message::{CliMessage, Command, RelayToCliMessage};
use super::state::gather_colony_state;

/// Configuration for relay connection
#[derive(Debug, Clone)]
pub struct RelayConfig {
    /// WebSocket URL (e.g., wss://api.colony.sh)
    pub url: String,
    /// Colony ID
    pub colony_id: String,
    /// Authentication token
    pub auth_token: String,
}

/// Relay client for WebSocket communication with app.colony.sh
pub struct RelayClient {
    config: RelayConfig,
    controller: Arc<Mutex<ColonyController>>,
    colony_root: PathBuf,
}

impl RelayClient {
    /// Create a new relay client
    pub fn new(config: RelayConfig, controller: ColonyController, colony_root: PathBuf) -> Self {
        Self {
            config,
            controller: Arc::new(Mutex::new(controller)),
            colony_root,
        }
    }

    /// Connect to relay service and start synchronization with auto-reconnect
    pub async fn connect(&self) -> ColonyResult<()> {
        let mut reconnect_delay = Duration::from_secs(2);
        let max_reconnect_delay = Duration::from_secs(60);

        loop {
            println!("🔗 Connecting to relay service at {}", self.config.url);

            match self.connect_once().await {
                Ok(_) => {
                    println!("🔌 Disconnected from relay service");
                    // Reset delay on clean disconnect
                    reconnect_delay = Duration::from_secs(2);
                }
                Err(e) => {
                    eprintln!("❌ Connection error: {}", e);
                }
            }

            // Attempt reconnection
            println!(
                "⏳ Reconnecting in {} seconds...",
                reconnect_delay.as_secs()
            );
            sleep(reconnect_delay).await;

            // Exponential backoff (up to max)
            reconnect_delay = std::cmp::min(reconnect_delay * 2, max_reconnect_delay);
        }
    }

    /// Single connection attempt (internal)
    async fn connect_once(&self) -> ColonyResult<()> {
        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&self.config.url)
            .await
            .map_err(|e| ColonyError::Colony(format!("Failed to connect to relay: {}", e)))?;

        println!("✓ Connected to relay service");

        let (mut write, mut read) = ws_stream.split();

        // Create channel for outbound messages (commands results, pongs, etc.)
        let (tx, mut rx) = mpsc::channel::<CliMessage>(100);

        // Send initial connect message
        let connect_msg = CliMessage::Connect {
            colony_id: self.config.colony_id.clone(),
            auth_token: self.config.auth_token.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        let msg_json = serde_json::to_string(&connect_msg)
            .map_err(|e| ColonyError::Colony(format!("Failed to serialize message: {}", e)))?;

        write
            .send(Message::Text(msg_json))
            .await
            .map_err(|e| ColonyError::Colony(format!("Failed to send connect message: {}", e)))?;

        println!("✓ Authenticated with relay service");
        println!("📡 Starting real-time synchronization...\n");

        // Clone necessary data for tasks
        let controller_clone = Arc::clone(&self.controller);
        let colony_root_clone = self.colony_root.clone();
        let config_clone = self.config.clone();
        let tx_clone = tx.clone();

        // Spawn state update task
        let state_handle = tokio::spawn(async move {
            if let Err(e) =
                Self::state_update_loop(tx_clone, controller_clone, colony_root_clone, config_clone)
                    .await
            {
                eprintln!("State update loop error: {}", e);
            }
        });

        // Spawn outbound message writer task
        let write_handle = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                let msg_json = match serde_json::to_string(&message) {
                    Ok(json) => json,
                    Err(e) => {
                        eprintln!("Failed to serialize outbound message: {}", e);
                        continue;
                    }
                };

                if let Err(e) = write.send(Message::Text(msg_json)).await {
                    eprintln!("Failed to send message: {}", e);
                    break;
                }
            }
        });

        // Handle incoming messages from relay
        let controller_clone = Arc::clone(&self.controller);
        let colony_root_clone = self.colony_root.clone();

        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    if let Err(e) = Self::handle_relay_message(
                        &text,
                        &controller_clone,
                        &colony_root_clone,
                        &tx,
                    )
                    .await
                    {
                        eprintln!("Error handling relay message: {}", e);
                    }
                }
                Ok(Message::Close(_)) => {
                    println!("🔌 Connection closed by relay service");
                    break;
                }
                Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        // Cleanup
        state_handle.abort();
        write_handle.abort();

        Ok(())
    }

    /// State update loop - sends colony state to relay every 5 seconds
    async fn state_update_loop(
        tx: mpsc::Sender<CliMessage>,
        controller: Arc<Mutex<ColonyController>>,
        colony_root: PathBuf,
        config: RelayConfig,
    ) -> ColonyResult<()> {
        let mut update_interval = interval(Duration::from_secs(5));

        loop {
            update_interval.tick().await;

            // Gather current state
            let controller_lock = controller.lock().await;
            let (agents, tasks, messages) =
                match gather_colony_state(&colony_root, &controller_lock).await {
                    Ok(state) => state,
                    Err(e) => {
                        eprintln!("Failed to gather colony state: {}", e);
                        continue;
                    }
                };
            drop(controller_lock);

            // Send state update via channel
            let state_update = CliMessage::StateUpdate {
                colony_id: config.colony_id.clone(),
                timestamp: Utc::now().timestamp(),
                agents,
                tasks,
                messages,
            };

            if let Err(e) = tx.send(state_update).await {
                eprintln!("Failed to send state update to channel: {}", e);
                break;
            }
        }

        Ok(())
    }

    /// Handle incoming message from relay
    async fn handle_relay_message(
        text: &str,
        controller: &Arc<Mutex<ColonyController>>,
        colony_root: &PathBuf,
        tx: &mpsc::Sender<CliMessage>,
    ) -> ColonyResult<()> {
        let message: RelayToCliMessage = serde_json::from_str(text)
            .map_err(|e| ColonyError::Colony(format!("Failed to parse relay message: {}", e)))?;

        match message {
            RelayToCliMessage::Command {
                request_id,
                command,
            } => {
                println!("📨 Received command: {:?}", command);
                let result = Self::execute_command(command, controller, colony_root).await;

                // Send command result back to relay
                let result_msg = match result {
                    Ok(output) => {
                        println!("✓ Command executed successfully: {}", output);
                        CliMessage::CommandResult {
                            request_id: request_id.clone(),
                            success: true,
                            output: Some(output),
                            error: None,
                        }
                    }
                    Err(e) => {
                        eprintln!("✗ Command execution failed: {}", e);
                        CliMessage::CommandResult {
                            request_id: request_id.clone(),
                            success: false,
                            output: None,
                            error: Some(e.to_string()),
                        }
                    }
                };

                // Send result via channel
                if let Err(e) = tx.send(result_msg).await {
                    eprintln!("Failed to send command result: {}", e);
                }
            }
            RelayToCliMessage::Ping => {
                // Respond with Pong
                if let Err(e) = tx.send(CliMessage::Pong).await {
                    eprintln!("Failed to send pong: {}", e);
                }
            }
            RelayToCliMessage::Connected { colony_id } => {
                println!("✓ Colony '{}' connected to relay", colony_id);
            }
            RelayToCliMessage::Error { message } => {
                eprintln!("❌ Relay error: {}", message);
            }
        }

        Ok(())
    }

    /// Execute a command received from relay
    async fn execute_command(
        command: Command,
        controller: &Arc<Mutex<ColonyController>>,
        colony_root: &PathBuf,
    ) -> ColonyResult<String> {
        match command {
            Command::SendMessage {
                to,
                content,
                message_type,
            } => {
                let msg_type = match message_type.as_str() {
                    "task" => crate::colony::messaging::MessageType::Task,
                    "question" => crate::colony::messaging::MessageType::Question,
                    "answer" => crate::colony::messaging::MessageType::Answer,
                    "completed" => crate::colony::messaging::MessageType::Completed,
                    "error" => crate::colony::messaging::MessageType::Error,
                    _ => crate::colony::messaging::MessageType::Info,
                };

                let message =
                    crate::colony::messaging::Message::new("relay", &to, content.clone(), msg_type);
                message.save(colony_root)?;

                Ok(format!("Message sent to {}", to))
            }
            Command::BroadcastMessage { content } => {
                let message = crate::colony::messaging::Message::new(
                    "relay",
                    "all",
                    content.clone(),
                    crate::colony::messaging::MessageType::Info,
                );
                message.save(colony_root)?;

                Ok("Broadcast message sent".to_string())
            }
            Command::CreateTask {
                title,
                description,
                assigned_to,
                priority,
            } => {
                use crate::colony::tasks::queue::TaskQueue;
                use crate::colony::tasks::{Task, TaskPriority};

                let task_id = format!("relay-{}", Utc::now().timestamp());
                let priority_enum = match priority.as_deref() {
                    Some("critical") => TaskPriority::Critical,
                    Some("high") => TaskPriority::High,
                    Some("medium") => TaskPriority::Medium,
                    _ => TaskPriority::Low,
                };

                let mut task = Task::new(task_id.clone(), title, description);
                task.assigned_to = assigned_to;
                task.priority = priority_enum;

                let queue = TaskQueue::new(colony_root);
                queue.create_task(task)?;

                Ok(format!("Task '{}' created", task_id))
            }
            Command::StopAgent { agent_id } => {
                // Stop the tmux pane for this agent
                let controller_lock = controller.lock().await;
                let session_name = controller_lock.config().session_name();
                drop(controller_lock);

                let output = tokio::process::Command::new("tmux")
                    .args(["kill-pane", "-t", &format!("{}:{}", session_name, agent_id)])
                    .output()
                    .await
                    .map_err(|e| ColonyError::Colony(format!("Failed to stop agent: {}", e)))?;

                if output.status.success() {
                    Ok(format!("Agent '{}' stopped", agent_id))
                } else {
                    Err(ColonyError::Colony(format!(
                        "Failed to stop agent: {}",
                        String::from_utf8_lossy(&output.stderr)
                    )))
                }
            }
            Command::StartAgent { agent_id } => {
                // Start an agent by recreating its tmux pane
                let controller_lock = controller.lock().await;
                let session_name = controller_lock.config().session_name();

                // Find the agent config
                let agent_config = controller_lock
                    .config()
                    .agents
                    .iter()
                    .find(|a| a.id == agent_id)
                    .ok_or_else(|| {
                        ColonyError::Colony(format!("Agent '{}' not found in config", agent_id))
                    })?
                    .clone();

                drop(controller_lock);

                // Create new pane for this agent
                let pane_name = format!("{}:{}", session_name, agent_id);

                // Split window to create new pane
                let output = tokio::process::Command::new("tmux")
                    .args([
                        "split-window",
                        "-t",
                        &session_name,
                        "-h", // horizontal split
                        "-P", // print pane info
                        "-F",
                        "#{pane_id}",
                    ])
                    .output()
                    .await
                    .map_err(|e| ColonyError::Colony(format!("Failed to create pane: {}", e)))?;

                if !output.status.success() {
                    return Err(ColonyError::Colony(format!(
                        "Failed to create pane: {}",
                        String::from_utf8_lossy(&output.stderr)
                    )));
                }

                // Rename the pane/window
                tokio::process::Command::new("tmux")
                    .args(["select-pane", "-t", &pane_name, "-T", &agent_id])
                    .output()
                    .await
                    .ok();

                Ok(format!("Agent '{}' started", agent_id))
            }
            Command::RestartAgent { agent_id } => {
                // Restart = Stop + Start
                let controller_lock = controller.lock().await;
                let session_name = controller_lock.config().session_name();

                // Find the agent config
                let _agent_config = controller_lock
                    .config()
                    .agents
                    .iter()
                    .find(|a| a.id == agent_id)
                    .ok_or_else(|| {
                        ColonyError::Colony(format!("Agent '{}' not found in config", agent_id))
                    })?
                    .clone();

                drop(controller_lock);

                // Stop the agent
                tokio::process::Command::new("tmux")
                    .args(["kill-pane", "-t", &format!("{}:{}", session_name, agent_id)])
                    .output()
                    .await
                    .ok();

                // Give it a moment to stop
                sleep(Duration::from_millis(500)).await;

                // Start it again - inline the StartAgent logic to avoid recursion
                // Split window to create new pane
                let output = tokio::process::Command::new("tmux")
                    .args([
                        "split-window",
                        "-t",
                        &session_name,
                        "-h", // horizontal split
                        "-P", // print pane info
                        "-F",
                        "#{pane_id}",
                    ])
                    .output()
                    .await
                    .map_err(|e| ColonyError::Colony(format!("Failed to create pane: {}", e)))?;

                if !output.status.success() {
                    return Err(ColonyError::Colony(format!(
                        "Failed to create pane: {}",
                        String::from_utf8_lossy(&output.stderr)
                    )));
                }

                // Rename the pane
                let pane_name = format!("{}:{}", session_name, agent_id);
                tokio::process::Command::new("tmux")
                    .args(["select-pane", "-t", &pane_name, "-T", &agent_id])
                    .output()
                    .await
                    .ok();

                Ok(format!("Agent '{}' restarted", agent_id))
            }
        }
    }
}
