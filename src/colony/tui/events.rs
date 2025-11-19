use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

/// Terminal events
#[derive(Debug, Clone, Copy)]
pub enum Event {
    /// A key was pressed
    Key(KeyEvent),
    /// Terminal was resized
    Resize,
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
                CrosstermEvent::Resize(_, _) => Ok(Event::Resize),
                _ => Ok(Event::Tick),
            }
        } else {
            Ok(Event::Tick)
        }
    }
}

/// Keyboard action
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Character input (for dialogs)
    CharInput(char),
    /// Backspace (for dialogs)
    Backspace,
    /// No action
    None,
}

impl Action {
    /// Convert a key event to an action, with context about whether we're in a dialog
    pub fn from_key(key: KeyEvent, in_dialog: bool) -> Self {
        // Handle control characters first
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return match key.code {
                KeyCode::Char('c') => Action::Quit,
                _ => Action::None,
            };
        }

        // Handle special keys
        match key.code {
            KeyCode::Esc => return Action::Cancel,
            KeyCode::Enter => return Action::Confirm,
            KeyCode::Backspace => {
                if in_dialog {
                    return Action::Backspace;
                }
            }
            _ => {}
        }

        // If we're in a dialog, capture printable characters
        if in_dialog {
            if let KeyCode::Char(c) = key.code {
                return Action::CharInput(c);
            }
            return Action::None;
        }

        // Normal mode key bindings
        match (key.code, key.modifiers) {
            // Quit
            (KeyCode::Char('q'), KeyModifiers::NONE) => Action::Quit,

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

            _ => Action::None,
        }
    }
}

impl From<KeyEvent> for Action {
    fn from(key: KeyEvent) -> Self {
        // Default to not in dialog
        Action::from_key(key, false)
    }
}
