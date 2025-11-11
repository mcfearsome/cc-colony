use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

/// Terminal events
#[derive(Debug, Clone, Copy)]
pub enum Event {
    /// A key was pressed
    Key(KeyEvent),
    /// Terminal was resized
    Resize(u16, u16),
    /// Tick event for periodic updates
    Tick,
}

/// Event handler
pub struct EventHandler {
    /// Tick rate in milliseconds
    tick_rate: u64,
}

impl EventHandler {
    /// Create a new event handler
    pub fn new(tick_rate: u64) -> Self {
        Self { tick_rate }
    }

    /// Poll for the next event (blocking with timeout)
    pub fn next(&self) -> Result<Event, std::io::Error> {
        // Poll for events with timeout
        if event::poll(Duration::from_millis(self.tick_rate))? {
            match event::read()? {
                CrosstermEvent::Key(key) => Ok(Event::Key(key)),
                CrosstermEvent::Resize(w, h) => Ok(Event::Resize(w, h)),
                _ => Ok(Event::Tick),
            }
        } else {
            Ok(Event::Tick)
        }
    }
}

/// Keyboard action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Quit the application
    Quit,
    /// Switch to tab by index (0-based)
    SwitchTab(usize),
    /// Scroll up
    ScrollUp,
    /// Scroll down
    ScrollDown,
    /// Page up
    PageUp,
    /// Page down
    PageDown,
    /// Show help
    ShowHelp,
    /// Refresh data
    Refresh,
    /// Broadcast message
    BroadcastMessage,
    /// Create task
    CreateTask,
    /// Send message to agent
    SendMessage,
    /// Cancel current action
    Cancel,
    /// Confirm current action
    Confirm,
    /// No action
    None,
}

impl From<KeyEvent> for Action {
    fn from(key: KeyEvent) -> Self {
        match (key.code, key.modifiers) {
            // Quit
            (KeyCode::Char('q'), KeyModifiers::NONE) => Action::Quit,
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => Action::Quit,
            (KeyCode::Esc, _) => Action::Cancel,

            // Tab switching
            (KeyCode::Char('1'), KeyModifiers::NONE) => Action::SwitchTab(0),
            (KeyCode::Char('2'), KeyModifiers::NONE) => Action::SwitchTab(1),
            (KeyCode::Char('3'), KeyModifiers::NONE) => Action::SwitchTab(2),
            (KeyCode::Char('4'), KeyModifiers::NONE) => Action::SwitchTab(3),
            (KeyCode::Tab, KeyModifiers::NONE) => Action::SwitchTab(usize::MAX), // Next tab
            (KeyCode::BackTab, KeyModifiers::SHIFT) => Action::SwitchTab(usize::MAX - 1), // Previous tab

            // Scrolling
            (KeyCode::Up, KeyModifiers::NONE) => Action::ScrollUp,
            (KeyCode::Down, KeyModifiers::NONE) => Action::ScrollDown,
            (KeyCode::PageUp, KeyModifiers::NONE) => Action::PageUp,
            (KeyCode::PageDown, KeyModifiers::NONE) => Action::PageDown,
            (KeyCode::Char('k'), KeyModifiers::NONE) => Action::ScrollUp,
            (KeyCode::Char('j'), KeyModifiers::NONE) => Action::ScrollDown,

            // Actions
            (KeyCode::Char('?'), KeyModifiers::NONE) => Action::ShowHelp,
            (KeyCode::Char('r'), KeyModifiers::NONE) => Action::Refresh,
            (KeyCode::Char('b'), KeyModifiers::NONE) => Action::BroadcastMessage,
            (KeyCode::Char('t'), KeyModifiers::NONE) => Action::CreateTask,
            (KeyCode::Char('m'), KeyModifiers::NONE) => Action::SendMessage,
            (KeyCode::Enter, KeyModifiers::NONE) => Action::Confirm,

            _ => Action::None,
        }
    }
}
