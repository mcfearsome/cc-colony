use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;
use std::path::Path;
use std::time::{Duration, Instant};

use super::data::ColonyData;
use super::events::{Action, Event, EventHandler};
use super::ui;

/// Tab in the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Agents,
    Tasks,
    Messages,
    State,
    Help,
}

impl Tab {
    pub fn index(&self) -> usize {
        match self {
            Tab::Agents => 0,
            Tab::Tasks => 1,
            Tab::Messages => 2,
            Tab::State => 3,
            Tab::Help => 4,
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Tab::Agents,
            1 => Tab::Tasks,
            2 => Tab::Messages,
            3 => Tab::State,
            4 => Tab::Help,
            _ => Tab::Agents,
        }
    }

    pub fn next(&self) -> Self {
        Self::from_index((self.index() + 1) % 5)
    }

    pub fn previous(&self) -> Self {
        Self::from_index((self.index() + 4) % 5)
    }
}

/// Dialog type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dialog {
    BroadcastMessage,
    CreateTask { step: usize },
    SendMessage { step: usize },
}

impl Dialog {
    /// Get the title for this dialog
    pub fn title(&self) -> &str {
        match self {
            Dialog::BroadcastMessage => "Broadcast Message to All Agents",
            Dialog::CreateTask { .. } => "Create New Task",
            Dialog::SendMessage { .. } => "Send Message to Agent",
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
        }
    }

    /// Get the total number of steps for this dialog
    pub fn total_steps(&self) -> usize {
        match self {
            Dialog::BroadcastMessage => 1,
            Dialog::CreateTask { .. } => 5,
            Dialog::SendMessage { .. } => 2,
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
}

impl App {
    /// Create a new application
    pub fn new(config_path: &Path) -> Result<Self, String> {
        let data = ColonyData::load(config_path)?;

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
        })
    }

    /// Handle an action
    pub fn handle_action(&mut self, action: Action) {
        // If we have an active dialog, handle dialog-specific actions
        if self.active_dialog.is_some() {
            match action {
                Action::Cancel => self.cancel_dialog(),
                Action::Confirm => self.confirm_dialog(),
                Action::CharInput(c) => self.input_buffer.push(c),
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
                } else if index < 5 {
                    self.current_tab = Tab::from_index(index);
                }
                self.scroll_position = 0;
            }
            Action::ScrollUp => {
                if self.scroll_position > 0 {
                    self.scroll_position -= 1;
                }
            }
            Action::ScrollDown => {
                self.scroll_position += 1;
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
            }
            Action::CreateTask => {
                self.active_dialog = Some(Dialog::CreateTask { step: 0 });
                self.input_buffer.clear();
                self.dialog_inputs.clear();
            }
            Action::SendMessage => {
                self.active_dialog = Some(Dialog::SendMessage { step: 0 });
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
