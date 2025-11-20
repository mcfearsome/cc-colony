use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::data::ColonyData;
use super::events::{Action, Event, EventHandler};
use super::ui;
use crate::colony::messaging::{Message, MessageType};

/// Tab in the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Agents,
    Tasks,
    Messages,
    State,
    Compose,
    Instructions,
    Config,
    Help,
}

impl Tab {
    pub fn index(&self) -> usize {
        match self {
            Tab::Agents => 0,
            Tab::Tasks => 1,
            Tab::Messages => 2,
            Tab::State => 3,
            Tab::Compose => 4,
            Tab::Instructions => 5,
            Tab::Config => 6,
            Tab::Help => 7,
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Tab::Agents,
            1 => Tab::Tasks,
            2 => Tab::Messages,
            3 => Tab::State,
            4 => Tab::Compose,
            5 => Tab::Instructions,
            6 => Tab::Config,
            7 => Tab::Help,
            _ => Tab::Agents, // Default to Agents for invalid indices
        }
    }

    pub fn next(&self) -> Self {
        Self::from_index((self.index() + 1) % 8)
    }

    pub fn previous(&self) -> Self {
        Self::from_index((self.index() + 7) % 8)
    }
}

/// Dialog type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dialog {
    BroadcastMessage,
    CreateTask { step: usize },
    SendMessage { step: usize },
    AddAgent { step: usize },
    AddExecutor { step: usize },
    AddMcpServer { step: usize },
    ConfigMenu,
}

impl Dialog {
    /// Get the title for this dialog
    pub fn title(&self) -> &str {
        match self {
            Dialog::BroadcastMessage => "Broadcast Message to All Agents",
            Dialog::CreateTask { .. } => "Create New Task",
            Dialog::SendMessage { .. } => "Send Message to Agent",
            Dialog::AddAgent { .. } => "Add New Agent",
            Dialog::AddExecutor { .. } => "Enable MCP Executor",
            Dialog::AddMcpServer { .. } => "Add MCP Server",
            Dialog::ConfigMenu => "Configuration Menu",
        }
    }

    /// Get the prompt for the current step
    pub fn prompt(&self) -> &str {
        match self {
            Dialog::BroadcastMessage => "Message:",
            Dialog::CreateTask { step } => match step {
                0 => "Task ID:",
                1 => "Title:",
                2 => "Description:",
                3 => "Assigned To (optional):",
                4 => "Priority (low/medium/high/critical):",
                _ => "",
            },
            Dialog::SendMessage { step } => match step {
                0 => "Agent ID:",
                1 => "Message:",
                _ => "",
            },
            Dialog::AddAgent { step } => match step {
                0 => "Agent ID:",
                1 => "Role:",
                2 => "Focus:",
                3 => "Model (sonnet/opus/haiku):",
                4 => "Worktree name (optional):",
                _ => "",
            },
            Dialog::AddExecutor { step } => match step {
                0 => "Enable executor? (y/n):",
                1 => "Select MCP servers (see MCP registry, comma-separated IDs):",
                _ => "",
            },
            Dialog::AddMcpServer { step } => match step {
                0 => "MCP Server ID (or select from registry with '?'):",
                1 => "Command:",
                2 => "Args (comma-separated, optional):",
                _ => "",
            },
            Dialog::ConfigMenu => "Select option: 1=Add Agent, 2=Add Executor, 3=Add MCP Server, ESC=Cancel",
        }
    }

    /// Get the total number of steps for this dialog
    pub fn total_steps(&self) -> usize {
        match self {
            Dialog::BroadcastMessage => 1,
            Dialog::CreateTask { .. } => 5,
            Dialog::SendMessage { .. } => 2,
            Dialog::AddAgent { .. } => 5,
            Dialog::AddExecutor { .. } => 2,
            Dialog::AddMcpServer { .. } => 3,
            Dialog::ConfigMenu => 1,
        }
    }
}

/// Main application state
pub struct App {
    /// Current active tab
    pub current_tab: Tab,
    /// Colony data
    pub data: ColonyData,
    /// Should the application quit?
    pub should_quit: bool,
    /// Last data refresh time
    last_refresh: Instant,
    /// Auto-refresh interval (seconds)
    refresh_interval: Duration,
    /// Scroll position for current view
    pub scroll_position: usize,
    /// Config file path
    config_path: String,
    /// Active dialog, if any
    pub active_dialog: Option<Dialog>,
    /// Input buffer for dialogs
    pub input_buffer: String,
    /// Collected inputs from multi-step dialogs
    pub dialog_inputs: Vec<String>,
    /// Status message to display
    pub status_message: Option<(String, bool)>, // (message, is_error)
    /// Available recipients (includes "all" + agent IDs)
    pub compose_recipients: Vec<String>,
    /// Selected recipient index
    pub compose_recipient_index: usize,
    /// Message content being composed (multiline buffer)
    pub compose_message: String,
    /// Which field has focus in Compose tab (0=message, 1=recipient)
    pub compose_focus: usize,
    /// Instructions input buffer (natural language orchestration)
    pub instructions: String,
}

impl App {
    /// Create a new application
    pub fn new(config_path: &Path) -> Result<Self, String> {
        let data = ColonyData::load(config_path)?;

        // Build recipient list: "all" first, then all agent IDs
        let mut recipients = vec!["all".to_string()];
        recipients.extend(data.agents.iter().map(|a| a.id.clone()));

        Ok(Self {
            current_tab: Tab::Agents,
            data,
            should_quit: false,
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_secs(2),
            scroll_position: 0,
            config_path: config_path.to_string_lossy().to_string(),
            active_dialog: None,
            input_buffer: String::new(),
            dialog_inputs: Vec::new(),
            status_message: None,
            compose_recipients: recipients,
            compose_recipient_index: 0,
            compose_message: String::new(),
            compose_focus: 0, // 0=message, 1=recipient
            instructions: String::new(),
        })
    }

    /// Handle an action
    pub fn handle_action(&mut self, action: Action) {
        // If we have an active dialog, handle dialog-specific actions
        if self.active_dialog.is_some() {
            match action {
                Action::Cancel => self.cancel_dialog(),
                Action::Confirm => self.confirm_dialog(),
                Action::InputChar(c) => self.input_buffer.push(c),
                Action::Backspace => {
                    self.input_buffer.pop();
                }
                _ => {}
            }
            return;
        }

        // Normal navigation actions
        match action {
            Action::Quit => self.should_quit = true,
            Action::SwitchTab(index) => {
                if index == usize::MAX {
                    // Next tab
                    self.current_tab = self.current_tab.next();
                } else if index == usize::MAX - 1 {
                    // Previous tab
                    self.current_tab = self.current_tab.previous();
                } else if index < 6 {
                    self.current_tab = Tab::from_index(index);
                }
                self.scroll_position = 0;
            }
            Action::ScrollUp => {
                if self.current_tab == Tab::Compose && self.compose_focus == 1 {
                    // Navigate recipient selection (focus==1 is recipient)
                    if self.compose_recipient_index > 0 {
                        self.compose_recipient_index -= 1;
                    }
                } else if self.scroll_position > 0 {
                    self.scroll_position -= 1;
                }
            }
            Action::ScrollDown => {
                if self.current_tab == Tab::Compose && self.compose_focus == 1 {
                    // Navigate recipient selection (focus==1 is recipient)
                    if self.compose_recipient_index < self.compose_recipients.len() - 1 {
                        self.compose_recipient_index += 1;
                    }
                } else {
                    self.scroll_position += 1;
                }
            }
            Action::PageUp => {
                if self.scroll_position >= 10 {
                    self.scroll_position -= 10;
                } else {
                    self.scroll_position = 0;
                }
            }
            Action::PageDown => {
                self.scroll_position += 10;
            }
            Action::ShowHelp => {
                self.current_tab = Tab::Help;
                self.scroll_position = 0;
            }
            Action::Refresh => {
                self.refresh_data();
                self.set_status("Data refreshed", false);
            }
            Action::BroadcastMessage => {
                self.active_dialog = Some(Dialog::BroadcastMessage);
                self.input_buffer.clear();
                self.dialog_inputs.clear();
                // Switch to compose tab with "all" pre-selected
                self.current_tab = Tab::Compose;
                self.compose_recipient_index = 0;
                self.compose_message.clear();
                self.compose_focus = 0;
                self.scroll_position = 0;
            }
            Action::SendMessage => {
                // Switch to compose tab
                self.current_tab = Tab::Compose;
                self.compose_recipient_index = 0;
                self.compose_message.clear();
                self.compose_focus = 0;
                self.scroll_position = 0;
            }
            Action::Cancel => {
                if self.current_tab == Tab::Compose {
                    self.compose_recipient_index = 0;
                    self.compose_message.clear();
                    self.compose_focus = 0;
                } else if self.current_tab == Tab::Instructions {
                    self.instructions.clear();
                }
            }
            Action::Confirm => {
                if self.current_tab == Tab::Compose {
                    self.send_message();
                } else if self.current_tab == Tab::Instructions {
                    self.execute_instructions();
                } else if self.current_tab == Tab::Config {
                    // Show config menu
                    self.active_dialog = Some(Dialog::ConfigMenu);
                    self.input_buffer.clear();
                }
            }
            Action::InputChar(c) => {
                if self.current_tab == Tab::Compose && self.compose_focus == 0 {
                    self.compose_message.push(c);
                } else if self.current_tab == Tab::Instructions {
                    self.instructions.push(c);
                }
            }
            Action::Backspace => {
                if self.current_tab == Tab::Compose && self.compose_focus == 0 {
                    self.compose_message.pop();
                } else if self.current_tab == Tab::Instructions {
                    self.instructions.pop();
                }
            }
            Action::NextField => {
                // Tab key switches focus between message field and recipient selector
                if self.current_tab == Tab::Compose {
                    self.compose_focus = (self.compose_focus + 1) % 2;
                }
            }
            Action::CreateTask => {
                self.active_dialog = Some(Dialog::CreateTask { step: 0 });
                self.input_buffer.clear();
                self.dialog_inputs.clear();
            }
            _ => {}
        }
    }

    /// Cancel the active dialog
    fn cancel_dialog(&mut self) {
        self.active_dialog = None;
        self.input_buffer.clear();
        self.dialog_inputs.clear();
        self.set_status("Cancelled", false);
    }

    /// Confirm the current dialog step
    fn confirm_dialog(&mut self) {
        if let Some(ref dialog) = self.active_dialog.clone() {
            match dialog {
                Dialog::BroadcastMessage => {
                    // Single-step dialog - execute immediately
                    let message = self.input_buffer.trim().to_string();
                    if !message.is_empty() {
                        self.execute_broadcast(message);
                    }
                    self.active_dialog = None;
                    self.input_buffer.clear();
                }
                Dialog::CreateTask { step } => {
                    // Multi-step dialog
                    self.dialog_inputs.push(self.input_buffer.trim().to_string());
                    self.input_buffer.clear();

                    if *step + 1 < dialog.total_steps() {
                        // Move to next step
                        self.active_dialog = Some(Dialog::CreateTask { step: step + 1 });
                    } else {
                        // Final step - execute
                        self.execute_create_task();
                        self.active_dialog = None;
                        self.dialog_inputs.clear();
                    }
                }
                Dialog::SendMessage { step } => {
                    // Multi-step dialog
                    self.dialog_inputs.push(self.input_buffer.trim().to_string());
                    self.input_buffer.clear();

                    if *step + 1 < dialog.total_steps() {
                        // Move to next step
                        self.active_dialog = Some(Dialog::SendMessage { step: step + 1 });
                    } else {
                        // Final step - execute
                        self.execute_send_message();
                        self.active_dialog = None;
                        self.dialog_inputs.clear();
                    }
                }
                Dialog::ConfigMenu => {
                    // Handle menu selection
                    let choice = self.input_buffer.trim().to_string();
                    self.input_buffer.clear();
                    self.active_dialog = None;

                    match choice.as_str() {
                        "1" => {
                            self.active_dialog = Some(Dialog::AddAgent { step: 0 });
                            self.dialog_inputs.clear();
                        }
                        "2" => {
                            self.active_dialog = Some(Dialog::AddExecutor { step: 0 });
                            self.dialog_inputs.clear();
                        }
                        "3" => {
                            self.active_dialog = Some(Dialog::AddMcpServer { step: 0 });
                            self.dialog_inputs.clear();
                        }
                        _ => {
                            self.set_status("Invalid choice", true);
                        }
                    }
                }
                Dialog::AddAgent { step } => {
                    // Multi-step dialog
                    self.dialog_inputs.push(self.input_buffer.trim().to_string());
                    self.input_buffer.clear();

                    if *step + 1 < dialog.total_steps() {
                        // Move to next step
                        self.active_dialog = Some(Dialog::AddAgent { step: step + 1 });
                    } else {
                        // Final step - execute
                        self.execute_add_agent();
                        self.active_dialog = None;
                        self.dialog_inputs.clear();
                    }
                }
                Dialog::AddExecutor { step } => {
                    // Multi-step dialog
                    self.dialog_inputs.push(self.input_buffer.trim().to_string());
                    self.input_buffer.clear();

                    if *step + 1 < dialog.total_steps() {
                        // Move to next step
                        self.active_dialog = Some(Dialog::AddExecutor { step: step + 1 });
                    } else {
                        // Final step - execute
                        self.execute_add_executor();
                        self.active_dialog = None;
                        self.dialog_inputs.clear();
                    }
                }
                Dialog::AddMcpServer { step } => {
                    // Multi-step dialog
                    self.dialog_inputs.push(self.input_buffer.trim().to_string());
                    self.input_buffer.clear();

                    if *step + 1 < dialog.total_steps() {
                        // Move to next step
                        self.active_dialog = Some(Dialog::AddMcpServer { step: step + 1 });
                    } else {
                        // Final step - execute
                        self.execute_add_mcp_server();
                        self.active_dialog = None;
                        self.dialog_inputs.clear();
                    }
                }
            }
        }
    }

    /// Execute broadcast message
    fn execute_broadcast(&mut self, message: String) {
        use crate::colony::{messaging, ColonyConfig, ColonyController};

        let config_path = Path::new(&self.config_path);
        match ColonyConfig::load(config_path) {
            Ok(config) => match ColonyController::new(config.clone()) {
                Ok(controller) => {
                    let msg = messaging::Message::new(
                        "operator",
                        "all",
                        message.clone(),
                        messaging::MessageType::Info,
                    );

                    match msg.save(controller.colony_root()) {
                        Ok(_) => {
                            // Also try to display in tmux
                            let session_name = config.session_name();
                            let escaped_msg = message.replace('#', "##");
                            let tmux_msg = format!("📢 BROADCAST: {}", escaped_msg);
                            let _ = std::process::Command::new("tmux")
                                .arg("display-message")
                                .arg("-t")
                                .arg(&session_name)
                                .arg(&tmux_msg)
                                .output();

                            self.set_status(
                                &format!("Broadcast sent: {}", message),
                                false,
                            );
                            self.refresh_data();
                        }
                        Err(e) => self.set_status(&format!("Error: {}", e), true),
                    }
                }
                Err(e) => self.set_status(&format!("Error: {}", e), true),
            },
            Err(e) => self.set_status(&format!("Error: {}", e), true),
        }
    }

    /// Execute create task
    fn execute_create_task(&mut self) {
        use crate::colony::{tasks::*, tasks::queue::TaskQueue, ColonyConfig, ColonyController};

        if self.dialog_inputs.len() < 5 {
            self.set_status("Not enough inputs for task creation", true);
            return;
        }

        let task_id = &self.dialog_inputs[0];
        let title = &self.dialog_inputs[1];
        let description = &self.dialog_inputs[2];
        let assigned_to = if self.dialog_inputs[3].is_empty() {
            None
        } else {
            Some(self.dialog_inputs[3].clone())
        };
        let priority = match self.dialog_inputs[4].to_lowercase().as_str() {
            "low" => TaskPriority::Low,
            "medium" | "" => TaskPriority::Medium,
            "high" => TaskPriority::High,
            "critical" => TaskPriority::Critical,
            _ => TaskPriority::Medium,
        };

        let config_path = Path::new(&self.config_path);
        match ColonyConfig::load(config_path) {
            Ok(config) => match ColonyController::new(config) {
                Ok(controller) => {
                    let mut task = Task::new(task_id.clone(), title.clone(), description.clone());
                    task.claimed_by = assigned_to;
                    task.priority = priority;

                    let queue = TaskQueue::new(controller.colony_root());
                    match queue.create_task(task) {
                        Ok(_) => {
                            self.set_status(&format!("Task created: {}", task_id), false);
                            self.refresh_data();
                        }
                        Err(e) => self.set_status(&format!("Error: {}", e), true),
                    }
                }
                Err(e) => self.set_status(&format!("Error: {}", e), true),
            },
            Err(e) => self.set_status(&format!("Error: {}", e), true),
        }
    }

    /// Execute send message
    fn execute_send_message(&mut self) {
        use crate::colony::{messaging, ColonyConfig, ColonyController};

        if self.dialog_inputs.len() < 2 {
            self.set_status("Not enough inputs for message", true);
            return;
        }

        let agent_id = &self.dialog_inputs[0];
        let message = &self.dialog_inputs[1];

        let config_path = Path::new(&self.config_path);
        match ColonyConfig::load(config_path) {
            Ok(config) => match ColonyController::new(config) {
                Ok(controller) => {
                    let msg = messaging::Message::new(
                        "operator",
                        agent_id,
                        message.clone(),
                        messaging::MessageType::Info,
                    );

                    match msg.save(controller.colony_root()) {
                        Ok(_) => {
                            self.set_status(
                                &format!("Message sent to {}: {}", agent_id, message),
                                false,
                            );
                            self.refresh_data();
                        }
                        Err(e) => self.set_status(&format!("Error: {}", e), true),
                    }
                }
                Err(e) => self.set_status(&format!("Error: {}", e), true),
            },
            Err(e) => self.set_status(&format!("Error: {}", e), true),
        }
    }

    /// Execute add agent
    fn execute_add_agent(&mut self) {
        use crate::colony::config::{AgentConfig, ColonyConfig};
        use std::collections::HashMap;

        if self.dialog_inputs.len() < 5 {
            self.set_status("Not enough inputs for add agent", true);
            return;
        }

        let agent_id = &self.dialog_inputs[0];
        let role = &self.dialog_inputs[1];
        let focus = &self.dialog_inputs[2];
        let model_input = &self.dialog_inputs[3];
        let worktree = if self.dialog_inputs[4].is_empty() {
            None
        } else {
            Some(self.dialog_inputs[4].clone())
        };

        // Map model shorthand to full model name
        let model = match model_input.to_lowercase().as_str() {
            "opus" => "claude-opus-4-20250514",
            "haiku" => "claude-3-5-haiku-20241022",
            "sonnet" | "" => "claude-sonnet-4-20250514",
            _ => model_input.as_str(),
        };

        let new_agent = AgentConfig {
            id: agent_id.clone(),
            role: role.clone(),
            focus: focus.clone(),
            model: model.to_string(),
            directory: None,
            worktree,
            env: None,
            mcp_servers: None,
            instructions: None,
            startup_prompt: None,
            capabilities: None,
            nudge: None,
        };

        // Load config, add agent, save
        let config_path = Path::new(&self.config_path);
        match ColonyConfig::load(config_path) {
            Ok(mut config) => {
                // Check for duplicate ID
                if config.agents.iter().any(|a| a.id == *agent_id) {
                    self.set_status(&format!("Agent ID '{}' already exists", agent_id), true);
                    return;
                }

                config.agents.push(new_agent);
                match config.save(config_path) {
                    Ok(_) => {
                        self.set_status(&format!("Agent '{}' added successfully. Restart colony to activate.", agent_id), false);
                        self.refresh_data();
                    }
                    Err(e) => self.set_status(&format!("Error saving config: {}", e), true),
                }
            }
            Err(e) => self.set_status(&format!("Error loading config: {}", e), true),
        }
    }

    /// Execute add executor
    fn execute_add_executor(&mut self) {
        use crate::colony::config::{ColonyConfig, ExecutorConfig};
        use crate::colony::mcp_registry::McpRegistry;
        use std::collections::HashMap;

        if self.dialog_inputs.len() < 2 {
            self.set_status("Not enough inputs for add executor", true);
            return;
        }

        let enable = self.dialog_inputs[0].to_lowercase();
        if enable != "y" && enable != "yes" {
            self.set_status("Executor not enabled", false);
            return;
        }

        let mcp_server_ids = &self.dialog_inputs[1];

        // Parse MCP server IDs
        let mut mcp_servers = HashMap::new();
        let mut server_ids = Vec::new();

        if !mcp_server_ids.is_empty() {
            for id in mcp_server_ids.split(',') {
                let id = id.trim();
                if let Some(server) = McpRegistry::get(id) {
                    mcp_servers.insert(server.id.clone(), server.config);
                    server_ids.push(id.to_string());
                } else {
                    self.set_status(&format!("Unknown MCP server ID: {}", id), true);
                    return;
                }
            }

            // Check for overlaps and show warnings
            let warnings = McpRegistry::detect_overlaps(&server_ids);
            if !warnings.is_empty() {
                let warning_msg = format!("Overlaps detected: {}", warnings.join("; "));
                self.set_status(&warning_msg, false);
                // Still proceed but warn the user
            }
        }

        let executor = ExecutorConfig {
            enabled: true,
            agent_id: "mcp-executor".to_string(),
            mcp_servers: if mcp_servers.is_empty() { None } else { Some(mcp_servers) },
            languages: vec!["typescript".to_string(), "python".to_string()],
        };

        // Load config, set executor, save
        let config_path = Path::new(&self.config_path);
        match ColonyConfig::load(config_path) {
            Ok(mut config) => {
                config.executor = Some(executor);
                match config.save(config_path) {
                    Ok(_) => {
                        self.set_status("Executor enabled successfully. Restart colony to activate.", false);
                        self.refresh_data();
                    }
                    Err(e) => self.set_status(&format!("Error saving config: {}", e), true),
                }
            }
            Err(e) => self.set_status(&format!("Error loading config: {}", e), true),
        }
    }

    /// Execute add MCP server
    fn execute_add_mcp_server(&mut self) {
        use crate::colony::config::{ColonyConfig, McpServerConfig};
        use crate::colony::mcp_registry::McpRegistry;
        use std::collections::HashMap;

        if self.dialog_inputs.len() < 3 {
            self.set_status("Not enough inputs for add MCP server", true);
            return;
        }

        let server_id = &self.dialog_inputs[0];

        // Check if user wants to select from registry
        if server_id == "?" {
            // Show available servers
            let servers = McpRegistry::all();
            let server_list: Vec<String> = servers.iter()
                .map(|s| format!("{}: {} - {}", s.id, s.name, s.description))
                .collect();
            self.set_status(&format!("Available MCP servers: {}", server_list.join(", ")), false);
            return;
        }

        // Try to get from registry first
        let server_config = if let Some(registry_server) = McpRegistry::get(server_id) {
            registry_server.config
        } else {
            // Manual configuration
            let command = &self.dialog_inputs[1];
            let args_str = &self.dialog_inputs[2];

            let args = if args_str.is_empty() {
                None
            } else {
                Some(args_str.split(',').map(|s| s.trim().to_string()).collect())
            };

            McpServerConfig {
                command: command.clone(),
                args,
                env: None,
            }
        };

        // Load config, add MCP server to executor, save
        let config_path = Path::new(&self.config_path);
        match ColonyConfig::load(config_path) {
            Ok(mut config) => {
                if let Some(ref mut executor) = config.executor {
                    let mut servers = executor.mcp_servers.take().unwrap_or_default();
                    servers.insert(server_id.clone(), server_config);
                    executor.mcp_servers = Some(servers);

                    match config.save(config_path) {
                        Ok(_) => {
                            self.set_status(&format!("MCP server '{}' added to executor. Restart colony to activate.", server_id), false);
                            self.refresh_data();
                        }
                        Err(e) => self.set_status(&format!("Error saving config: {}", e), true),
                    }
                } else {
                    self.set_status("Executor not enabled. Enable executor first.", true);
                }
            }
            Err(e) => self.set_status(&format!("Error loading config: {}", e), true),
        }
    }

    /// Set a status message
    fn set_status(&mut self, message: &str, is_error: bool) {
        self.status_message = Some((message.to_string(), is_error));
    }

    /// Clear status message
    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    /// Refresh colony data from disk
    pub fn refresh_data(&mut self) {
        if let Ok(data) = ColonyData::load(Path::new(&self.config_path)) {
            // Rebuild recipient list in case agents changed
            let mut recipients = vec!["all".to_string()];
            recipients.extend(data.agents.iter().map(|a| a.id.clone()));

            // Clamp recipient index if agents were removed
            if self.compose_recipient_index >= recipients.len() {
                self.compose_recipient_index = 0;
            }

            self.compose_recipients = recipients;
            self.data = data;
            self.last_refresh = Instant::now();
        }
    }

    /// Check if data should be auto-refreshed
    pub fn should_auto_refresh(&self) -> bool {
        self.last_refresh.elapsed() >= self.refresh_interval
    }

    /// Update the application (called on each tick)
    pub fn update(&mut self) {
        if self.should_auto_refresh() {
            self.refresh_data();
        }
    }

    /// Send the composed message
    fn send_message(&mut self) {
        if self.compose_message.is_empty() {
            return;
        }

        // Get the recipient from the selected index
        let recipient = &self.compose_recipients[self.compose_recipient_index];

        // Use absolute path to .colony directory
        let colony_root = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".colony");

        // Create message with sender as "orchestrator"
        let message = Message::new(
            "orchestrator",
            recipient,
            self.compose_message.clone(),
            MessageType::Info,
        );

        // Try to save the message
        if let Err(_e) = message.save(&colony_root) {
            // Silently fail - we're in TUI mode so we can't show errors easily
            // The message just won't be sent
        }

        // Clear fields after sending
        self.compose_recipient_index = 0;
        self.compose_message.clear();
        self.compose_focus = 0;

        // Refresh data to show the new message
        self.refresh_data();
    }

    /// Execute natural language instructions by routing to agents
    fn execute_instructions(&mut self) {
        if self.instructions.trim().is_empty() {
            return;
        }

        let colony_root = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(".colony");

        // Simple routing: broadcast to all agents as a task
        // Future: Could use LLM to parse and route to specific agents
        let instruction = self.instructions.clone();

        // Send as broadcast task
        let message = Message::new(
            "orchestrator",
            "all",
            format!("INSTRUCTION: {}", instruction),
            MessageType::Task,
        );

        let _ = message.save(&colony_root);

        // Clear instructions after sending
        self.instructions.clear();

        // Refresh to show new message
        self.refresh_data();
    }
}

/// Run the TUI application
pub fn run_tui(config_path: &Path) -> Result<(), String> {
    // Setup terminal
    enable_raw_mode().map_err(|e| format!("Failed to enable raw mode: {}", e))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)
        .map_err(|e| format!("Failed to enter alternate screen: {}", e))?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal =
        Terminal::new(backend).map_err(|e| format!("Failed to create terminal: {}", e))?;

    // Create app
    let mut app = App::new(config_path)?;

    // Create event handler (250ms tick rate)
    let event_handler = EventHandler::new(250);

    // Main loop
    let result = run_app(&mut terminal, &mut app, event_handler);

    // Restore terminal
    disable_raw_mode().map_err(|e| format!("Failed to disable raw mode: {}", e))?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .map_err(|e| format!("Failed to leave alternate screen: {}", e))?;
    terminal
        .show_cursor()
        .map_err(|e| format!("Failed to show cursor: {}", e))?;

    result
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    event_handler: EventHandler,
) -> Result<(), String> {
    loop {
        // Draw UI
        terminal
            .draw(|f| ui::render(f, app))
            .map_err(|e| format!("Failed to draw: {}", e))?;

        // Handle events
        match event_handler.next() {
            Ok(Event::Key(key)) => {
                let in_dialog = app.active_dialog.is_some();
                let action = Action::from_key(key, in_dialog);
                use crossterm::event::{KeyCode, KeyModifiers};

                // In Compose or Instructions tab, handle text input differently
                let action = if app.current_tab == Tab::Compose {
                    match (key.code, key.modifiers) {
                        (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => Action::InputChar(c),
                        (KeyCode::Backspace, _) => Action::Backspace,
                        (KeyCode::Tab, _) => Action::NextField,
                        (KeyCode::Esc, _) => Action::Cancel,
                        // Enter adds newline in message field, sends when in recipient field
                        (KeyCode::Enter, _) if app.compose_focus == 0 => Action::InputChar('\n'),
                        (KeyCode::Enter, _) if app.compose_focus == 1 => Action::Confirm,
                        (KeyCode::Up, _) => Action::ScrollUp,
                        (KeyCode::Down, _) => Action::ScrollDown,
                        _ => Action::from(key),
                    }
                } else if app.current_tab == Tab::Instructions {
                    match (key.code, key.modifiers) {
                        (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => Action::InputChar(c),
                        (KeyCode::Backspace, _) => Action::Backspace,
                        (KeyCode::Enter, _) => Action::Confirm,
                        (KeyCode::Esc, _) => Action::Cancel,
                        _ => Action::from(key),
                    }
                } else {
                    Action::from(key)
                };

                app.handle_action(action);
            }
            Ok(Event::Resize) => {
                // Terminal was resized, will be handled on next draw
            }
            Ok(Event::Tick) => {
                app.update();
            }
            Err(e) => {
                return Err(format!("Event error: {}", e));
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}
