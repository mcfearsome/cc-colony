use super::events::TelemetryEvent;
use crate::colony::config::TelemetryConfig;
use crate::error::ColonyResult;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};

/// Telemetry client for collecting and sending events
#[derive(Clone)]
pub struct TelemetryClient {
    config: Arc<Mutex<TelemetryConfig>>,
    session_id: String,
    event_sender: mpsc::UnboundedSender<TelemetryEvent>,
}

impl TelemetryClient {
    /// Create a new telemetry client
    pub fn new(config: TelemetryConfig) -> Self {
        let session_id = uuid::Uuid::new_v4().to_string();
        let (tx, rx) = mpsc::unbounded_channel();

        let client = Self {
            config: Arc::new(Mutex::new(config)),
            session_id,
            event_sender: tx,
        };

        // Spawn background worker to batch and send events
        let worker_config = client.config.clone();
        tokio::spawn(async move {
            Self::event_worker(rx, worker_config).await;
        });

        client
    }

    /// Check if telemetry is enabled
    pub async fn is_enabled(&self) -> bool {
        self.config.lock().await.enabled
    }

    /// Get session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get anonymous ID
    pub async fn anonymous_id(&self) -> String {
        self.config.lock().await.get_or_create_anonymous_id()
    }

    /// Track an event (non-blocking)
    pub async fn track(&self, event: TelemetryEvent) {
        // Only send if telemetry is enabled
        if !self.is_enabled().await {
            return;
        }

        // Send event to background worker (non-blocking)
        let _ = self.event_sender.send(event);
    }

    /// Track a command invocation
    pub async fn track_command(&self, command: &str, duration_ms: Option<u64>) {
        if !self.is_enabled().await {
            return;
        }

        let anonymous_id = self.anonymous_id().await;
        let event = TelemetryEvent::command_invoked(
            self.session_id.clone(),
            anonymous_id,
            command,
            duration_ms,
        );

        self.track(event).await;
    }

    /// Track colony started
    pub async fn track_colony_started(&self, agent_count: usize, has_executor: bool) {
        if !self.is_enabled().await {
            return;
        }

        let anonymous_id = self.anonymous_id().await;
        let event = TelemetryEvent::colony_started(
            self.session_id.clone(),
            anonymous_id,
            agent_count,
            has_executor,
        );

        self.track(event).await;
    }

    /// Track feature usage
    pub async fn track_feature(&self, feature: &str) {
        if !self.is_enabled().await {
            return;
        }

        let anonymous_id = self.anonymous_id().await;
        let event = TelemetryEvent::feature_used(self.session_id.clone(), anonymous_id, feature);

        self.track(event).await;
    }

    /// Track an error (without PII)
    pub async fn track_error(&self, error_type: &str, command: &str) {
        if !self.is_enabled().await {
            return;
        }

        let anonymous_id = self.anonymous_id().await;
        let event = TelemetryEvent::error_occurred(
            self.session_id.clone(),
            anonymous_id,
            error_type,
            command,
        );

        self.track(event).await;
    }

    /// Background worker that batches and sends events
    async fn event_worker(
        mut rx: mpsc::UnboundedReceiver<TelemetryEvent>,
        config: Arc<Mutex<TelemetryConfig>>,
    ) {
        let mut events_buffer = Vec::new();
        let mut tick = interval(Duration::from_secs(30)); // Batch every 30 seconds

        loop {
            tokio::select! {
                // Receive new events
                Some(event) = rx.recv() => {
                    events_buffer.push(event);

                    // If buffer gets large, send immediately
                    if events_buffer.len() >= 10 {
                        Self::send_batch(&events_buffer, &config).await;
                        events_buffer.clear();
                    }
                }
                // Periodic batch send
                _ = tick.tick() => {
                    if !events_buffer.is_empty() {
                        Self::send_batch(&events_buffer, &config).await;
                        events_buffer.clear();
                    }
                }
                // Channel closed, send remaining events and exit
                else => {
                    if !events_buffer.is_empty() {
                        Self::send_batch(&events_buffer, &config).await;
                    }
                    break;
                }
            }
        }
    }

    /// Send a batch of events to the telemetry endpoint
    async fn send_batch(events: &[TelemetryEvent], config: &Arc<Mutex<TelemetryConfig>>) {
        let config_guard = config.lock().await;

        // Double-check telemetry is enabled
        if !config_guard.enabled {
            return;
        }

        let endpoint = config_guard.endpoint_url();
        drop(config_guard); // Release lock before making HTTP request

        // Send events to endpoint (gracefully handle failures)
        if let Err(e) = Self::send_events_http(&endpoint, events).await {
            // Log error but don't crash the CLI
            eprintln!(
                "Failed to send telemetry (this won't affect CLI functionality): {}",
                e
            );
        }
    }

    /// Send events via HTTP POST
    async fn send_events_http(endpoint: &str, events: &[TelemetryEvent]) -> ColonyResult<()> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| crate::error::ColonyError::Other(e.to_string()))?;

        let response = client
            .post(endpoint)
            .json(&events)
            .send()
            .await
            .map_err(|e| crate::error::ColonyError::Other(e.to_string()))?;

        if !response.status().is_success() {
            return Err(crate::error::ColonyError::Other(format!(
                "Telemetry endpoint returned status: {}",
                response.status()
            )));
        }

        Ok(())
    }

    /// Enable telemetry and update config
    pub async fn enable(&self, config_path: &std::path::Path) -> ColonyResult<()> {
        let mut config_guard = self.config.lock().await;
        config_guard.enabled = true;
        config_guard.get_or_create_anonymous_id();

        // Load full config, update telemetry section, and save
        let mut colony_config = crate::colony::config::ColonyConfig::load(config_path)?;
        colony_config.telemetry = config_guard.clone();
        colony_config.save(config_path)?;

        Ok(())
    }

    /// Disable telemetry and update config
    pub async fn disable(&self, config_path: &std::path::Path) -> ColonyResult<()> {
        let mut config_guard = self.config.lock().await;
        config_guard.enabled = false;

        // Load full config, update telemetry section, and save
        let mut colony_config = crate::colony::config::ColonyConfig::load(config_path)?;
        colony_config.telemetry = config_guard.clone();
        colony_config.save(config_path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_disabled_telemetry_does_not_send() {
        let config = TelemetryConfig {
            enabled: false,
            anonymous_id: None,
            endpoint: None,
        };

        let client = TelemetryClient::new(config);
        assert!(!client.is_enabled().await);

        // This should not panic even though telemetry is disabled
        client.track_command("test", None).await;
    }

    #[tokio::test]
    async fn test_enabled_telemetry() {
        let config = TelemetryConfig {
            enabled: true,
            anonymous_id: Some("test-user".to_string()),
            endpoint: Some("http://localhost:9999/telemetry".to_string()),
        };

        let client = TelemetryClient::new(config);
        assert!(client.is_enabled().await);
        assert_eq!(client.anonymous_id().await, "test-user");
    }
}
