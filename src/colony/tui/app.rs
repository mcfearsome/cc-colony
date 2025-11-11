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
    Help,
}

impl Tab {
    pub fn name(&self) -> &'static str {
        match self {
            Tab::Agents => "Agents",
            Tab::Tasks => "Tasks",
            Tab::Messages => "Messages",
            Tab::Help => "Help",
        }
    }

    pub fn index(&self) -> usize {
        match self {
            Tab::Agents => 0,
            Tab::Tasks => 1,
            Tab::Messages => 2,
            Tab::Help => 3,
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Tab::Agents,
            1 => Tab::Tasks,
            2 => Tab::Messages,
            3 => Tab::Help,
            _ => Tab::Agents,
        }
    }

    pub fn next(&self) -> Self {
        Self::from_index((self.index() + 1) % 4)
    }

    pub fn previous(&self) -> Self {
        Self::from_index((self.index() + 3) % 4)
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
        })
    }

    /// Handle an action
    pub fn handle_action(&mut self, action: Action) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::SwitchTab(index) => {
                if index == usize::MAX {
                    // Next tab
                    self.current_tab = self.current_tab.next();
                } else if index == usize::MAX - 1 {
                    // Previous tab
                    self.current_tab = self.current_tab.previous();
                } else if index < 4 {
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
            }
            Action::BroadcastMessage => {
                // TODO: Show broadcast dialog
            }
            Action::CreateTask => {
                // TODO: Show create task dialog
            }
            Action::SendMessage => {
                // TODO: Show send message dialog
            }
            _ => {}
        }
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
                let action = Action::from(key);
                app.handle_action(action);
            }
            Ok(Event::Resize(_, _)) => {
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
