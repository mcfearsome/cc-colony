use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

impl LogLevel {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "debug" => Some(LogLevel::Debug),
            "info" => Some(LogLevel::Info),
            "warn" | "warning" => Some(LogLevel::Warn),
            "error" => Some(LogLevel::Error),
            _ => None,
        }
    }

    pub fn color(&self) -> colored::Color {
        use colored::Color;
        match self {
            LogLevel::Debug => Color::BrightBlack,
            LogLevel::Info => Color::Blue,
            LogLevel::Warn => Color::Yellow,
            LogLevel::Error => Color::Red,
        }
    }
}

/// Structured log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub agent_id: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

impl LogEntry {
    pub fn new(level: LogLevel, agent_id: String, message: String) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            agent_id,
            message,
            context: None,
        }
    }

    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }

    /// Format as JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Format as human-readable text
    pub fn to_text(&self, colorize: bool) -> String {
        use colored::Colorize;

        let timestamp = self.timestamp.format("%Y-%m-%d %H:%M:%S%.3f");
        let level_str = format!("{:5}", self.level.to_string());
        let agent_str = format!("[{}]", self.agent_id);

        if colorize {
            format!(
                "{} {} {} {}",
                timestamp.to_string().dimmed(),
                level_str.color(self.level.color()),
                agent_str.cyan(),
                self.message
            )
        } else {
            format!("{} {} {} {}", timestamp, level_str, agent_str, self.message)
        }
    }

    /// Parse from JSON line
    pub fn from_json(line: &str) -> Option<Self> {
        serde_json::from_str(line).ok()
    }

    /// Parse from text line (best effort)
    pub fn from_text(line: &str) -> Option<Self> {
        // Try to parse timestamp and level
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            return None;
        }

        // Look for log level keywords
        let level = if line.contains("ERROR") {
            LogLevel::Error
        } else if line.contains("WARN") {
            LogLevel::Warn
        } else if line.contains("INFO") {
            LogLevel::Info
        } else if line.contains("DEBUG") {
            LogLevel::Debug
        } else {
            LogLevel::Info // default
        };

        Some(LogEntry {
            timestamp: Utc::now(), // Can't reliably parse from text
            level,
            agent_id: "unknown".to_string(),
            message: line.to_string(),
            context: None,
        })
    }
}

/// Log filter criteria
#[derive(Debug, Clone)]
pub struct LogFilter {
    pub min_level: Option<LogLevel>,
    pub agent_id: Option<String>,
    pub pattern: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
}

impl Default for LogFilter {
    fn default() -> Self {
        Self {
            min_level: None,
            agent_id: None,
            pattern: None,
            since: None,
            until: None,
        }
    }
}

impl LogFilter {
    pub fn matches(&self, entry: &LogEntry) -> bool {
        // Filter by level
        if let Some(min_level) = self.min_level {
            if entry.level < min_level {
                return false;
            }
        }

        // Filter by agent
        if let Some(ref agent_id) = self.agent_id {
            if &entry.agent_id != agent_id {
                return false;
            }
        }

        // Filter by pattern
        if let Some(ref pattern) = self.pattern {
            if !entry.message.contains(pattern) {
                return false;
            }
        }

        // Filter by time range
        if let Some(since) = self.since {
            if entry.timestamp < since {
                return false;
            }
        }

        if let Some(until) = self.until {
            if entry.timestamp > until {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("debug"), Some(LogLevel::Debug));
        assert_eq!(LogLevel::from_str("INFO"), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("warn"), Some(LogLevel::Warn));
        assert_eq!(LogLevel::from_str("ERROR"), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("invalid"), None);
    }

    #[test]
    fn test_log_entry_json() {
        let entry = LogEntry::new(
            LogLevel::Info,
            "agent-1".to_string(),
            "Test message".to_string(),
        );

        let json = entry.to_json();
        let parsed = LogEntry::from_json(&json);

        assert!(parsed.is_some());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.level, LogLevel::Info);
        assert_eq!(parsed.agent_id, "agent-1");
        assert_eq!(parsed.message, "Test message");
    }
}
