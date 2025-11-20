use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of telemetry event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TelemetryEventType {
    /// CLI command was invoked
    CommandInvoked,
    /// Colony was started
    ColonyStarted,
    /// Colony was stopped
    ColonyStopped,
    /// Agent lifecycle event
    AgentLifecycle,
    /// Authentication event
    AuthEvent,
    /// Feature usage event
    FeatureUsed,
    /// Error occurred
    ErrorOccurred,
    /// Session started
    SessionStarted,
    /// Session ended
    SessionEnded,
}

/// A telemetry event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    /// Event ID (unique)
    pub event_id: String,
    /// Session ID (persists across related events)
    pub session_id: String,
    /// Anonymous user ID
    pub anonymous_id: String,
    /// Event type
    pub event_type: TelemetryEventType,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Event properties (no PII)
    pub properties: HashMap<String, String>,
    /// CLI version
    pub cli_version: String,
    /// Operating system
    pub os: String,
    /// Architecture
    pub arch: String,
}

impl TelemetryEvent {
    /// Create a new telemetry event
    pub fn new(
        session_id: String,
        anonymous_id: String,
        event_type: TelemetryEventType,
        properties: HashMap<String, String>,
    ) -> Self {
        Self {
            event_id: uuid::Uuid::new_v4().to_string(),
            session_id,
            anonymous_id,
            event_type,
            timestamp: Utc::now(),
            properties,
            cli_version: env!("CARGO_PKG_VERSION").to_string(),
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
        }
    }

    /// Create a command invocation event
    pub fn command_invoked(
        session_id: String,
        anonymous_id: String,
        command: &str,
        duration_ms: Option<u64>,
    ) -> Self {
        let mut properties = HashMap::new();
        properties.insert("command".to_string(), command.to_string());
        if let Some(duration) = duration_ms {
            properties.insert("duration_ms".to_string(), duration.to_string());
        }

        Self::new(
            session_id,
            anonymous_id,
            TelemetryEventType::CommandInvoked,
            properties,
        )
    }

    /// Create a colony started event
    pub fn colony_started(
        session_id: String,
        anonymous_id: String,
        agent_count: usize,
        has_executor: bool,
    ) -> Self {
        let mut properties = HashMap::new();
        properties.insert("agent_count".to_string(), agent_count.to_string());
        properties.insert("has_executor".to_string(), has_executor.to_string());

        Self::new(
            session_id,
            anonymous_id,
            TelemetryEventType::ColonyStarted,
            properties,
        )
    }

    /// Create an agent lifecycle event
    pub fn agent_lifecycle(session_id: String, anonymous_id: String, action: &str) -> Self {
        let mut properties = HashMap::new();
        properties.insert("action".to_string(), action.to_string());

        Self::new(
            session_id,
            anonymous_id,
            TelemetryEventType::AgentLifecycle,
            properties,
        )
    }

    /// Create an auth event
    pub fn auth_event(
        session_id: String,
        anonymous_id: String,
        auth_type: &str,
        action: &str,
    ) -> Self {
        let mut properties = HashMap::new();
        properties.insert("auth_type".to_string(), auth_type.to_string());
        properties.insert("action".to_string(), action.to_string());

        Self::new(
            session_id,
            anonymous_id,
            TelemetryEventType::AuthEvent,
            properties,
        )
    }

    /// Create a feature usage event
    pub fn feature_used(session_id: String, anonymous_id: String, feature: &str) -> Self {
        let mut properties = HashMap::new();
        properties.insert("feature".to_string(), feature.to_string());

        Self::new(
            session_id,
            anonymous_id,
            TelemetryEventType::FeatureUsed,
            properties,
        )
    }

    /// Create an error event (without PII)
    pub fn error_occurred(
        session_id: String,
        anonymous_id: String,
        error_type: &str,
        command: &str,
    ) -> Self {
        let mut properties = HashMap::new();
        properties.insert("error_type".to_string(), error_type.to_string());
        properties.insert("command".to_string(), command.to_string());

        Self::new(
            session_id,
            anonymous_id,
            TelemetryEventType::ErrorOccurred,
            properties,
        )
    }

    /// Create a session started event
    pub fn session_started(session_id: String, anonymous_id: String) -> Self {
        Self::new(
            session_id,
            anonymous_id,
            TelemetryEventType::SessionStarted,
            HashMap::new(),
        )
    }

    /// Create a session ended event
    pub fn session_ended(session_id: String, anonymous_id: String, duration_ms: u64) -> Self {
        let mut properties = HashMap::new();
        properties.insert("duration_ms".to_string(), duration_ms.to_string());

        Self::new(
            session_id,
            anonymous_id,
            TelemetryEventType::SessionEnded,
            properties,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_command_event() {
        let event = TelemetryEvent::command_invoked(
            "session-123".to_string(),
            "user-456".to_string(),
            "start",
            Some(1500),
        );

        assert_eq!(event.event_type, TelemetryEventType::CommandInvoked);
        assert_eq!(event.session_id, "session-123");
        assert_eq!(event.anonymous_id, "user-456");
        assert_eq!(event.properties.get("command"), Some(&"start".to_string()));
        assert_eq!(
            event.properties.get("duration_ms"),
            Some(&"1500".to_string())
        );
    }

    #[test]
    fn test_create_colony_started_event() {
        let event = TelemetryEvent::colony_started(
            "session-123".to_string(),
            "user-456".to_string(),
            3,
            true,
        );

        assert_eq!(event.event_type, TelemetryEventType::ColonyStarted);
        assert_eq!(event.properties.get("agent_count"), Some(&"3".to_string()));
        assert_eq!(
            event.properties.get("has_executor"),
            Some(&"true".to_string())
        );
    }
}
