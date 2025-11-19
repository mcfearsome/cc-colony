use crate::colony::controller::ColonyController;
use crate::error::{ColonyError, ColonyResult};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
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

    /// Connect to relay service and start synchronization
    pub async fn connect(&self) -> ColonyResult<()> {
        println!("🔗 Connecting to relay service at {}", self.config.url);

        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&self.config.url)
            .await
            .map_err(|e| ColonyError::Colony(format!("Failed to connect to relay: {}", e)))?;

        println!("✓ Connected to relay service");

        let (mut write, mut read) = ws_stream.split();

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

        // Spawn state update task
        let write_handle = tokio::spawn(async move {
            if let Err(e) = Self::state_update_loop(write, controller_clone, colony_root_clone, config_clone).await {
                eprintln!("State update loop error: {}", e);
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
        write_handle.abort();

        Ok(())
    }

    /// State update loop - sends colony state to relay every 5 seconds
    async fn state_update_loop(
        mut write: futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<
                tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
            >,
            Message,
        >,
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

            // Send state update
            let state_update = CliMessage::StateUpdate {
                colony_id: config.colony_id.clone(),
                timestamp: Utc::now().timestamp(),
                agents,
                tasks,
                messages,
            };

            let msg_json = match serde_json::to_string(&state_update) {
                Ok(json) => json,
                Err(e) => {
                    eprintln!("Failed to serialize state update: {}", e);
                    continue;
                }
            };

            if let Err(e) = write.send(Message::Text(msg_json)).await {
                eprintln!("Failed to send state update: {}", e);
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

                // TODO: Send command result back to relay
                match result {
                    Ok(output) => {
                        println!("✓ Command executed successfully: {}", output);
                    }
                    Err(e) => {
                        eprintln!("✗ Command execution failed: {}", e);
                    }
                }

                // Note: We'd need to send CommandResult back through the write half
                // For now, just logging
            }
            RelayToCliMessage::Ping => {
                // TODO: Send Pong back
                // Need to restructure to have access to write half here
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
                use crate::colony::tasks::{Task, TaskPriority};
                use crate::colony::tasks::queue::TaskQueue;

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
                // TODO: Implement agent starting
                // This is more complex as it requires setting up the pane properly
                Ok(format!(
                    "Starting agents from relay not yet implemented for '{}'",
                    agent_id
                ))
            }
            Command::RestartAgent { agent_id } => {
                // TODO: Implement agent restart
                Ok(format!(
                    "Restarting agents from relay not yet implemented for '{}'",
                    agent_id
                ))
            }
        }
    }
}
