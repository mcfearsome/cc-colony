use super::types::{Metric, MetricType};
use crate::error::ColonyResult;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Metrics collector that stores and manages metrics
#[derive(Clone)]
pub struct MetricsCollector {
    metrics: Arc<Mutex<HashMap<String, Metric>>>,
    retention_period: Duration,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(HashMap::new())),
            retention_period: Duration::hours(24), // Default 24 hours retention
        }
    }

    /// Create with custom retention period
    pub fn with_retention(retention_period: Duration) -> Self {
        Self {
            metrics: Arc::new(Mutex::new(HashMap::new())),
            retention_period,
        }
    }

    /// Register a new metric
    pub fn register_metric(
        &self,
        name: String,
        metric_type: MetricType,
        description: String,
        unit: Option<String>,
    ) -> ColonyResult<()> {
        let mut metrics = self.metrics.lock().unwrap();
        if metrics.contains_key(&name) {
            return Ok(()); // Already registered
        }

        let mut metric = Metric::new(name.clone(), metric_type, description);
        if let Some(u) = unit {
            metric = metric.with_unit(u);
        }

        metrics.insert(name, metric);
        Ok(())
    }

    /// Record a metric value
    pub fn record(
        &self,
        name: &str,
        value: f64,
        labels: HashMap<String, String>,
    ) -> ColonyResult<()> {
        let mut metrics = self.metrics.lock().unwrap();
        if let Some(metric) = metrics.get_mut(name) {
            metric.record(value, labels);
        }
        // Silently ignore if metric doesn't exist
        Ok(())
    }

    /// Record a metric value with no labels
    pub fn record_simple(&self, name: &str, value: f64) -> ColonyResult<()> {
        self.record(name, value, HashMap::new())
    }

    /// Increment a counter by 1
    pub fn increment(&self, name: &str, labels: HashMap<String, String>) -> ColonyResult<()> {
        self.record(name, 1.0, labels)
    }

    /// Increment a counter with no labels
    pub fn increment_simple(&self, name: &str) -> ColonyResult<()> {
        self.record_simple(name, 1.0)
    }

    /// Get a metric by name
    pub fn get_metric(&self, name: &str) -> Option<Metric> {
        let metrics = self.metrics.lock().unwrap();
        metrics.get(name).cloned()
    }

    /// List all registered metrics
    pub fn list_metrics(&self) -> Vec<Metric> {
        let metrics = self.metrics.lock().unwrap();
        metrics.values().cloned().collect()
    }

    /// Get metric names
    pub fn metric_names(&self) -> Vec<String> {
        let metrics = self.metrics.lock().unwrap();
        metrics.keys().cloned().collect()
    }

    /// Prune old data points from all metrics
    pub fn prune_old_data(&self) -> ColonyResult<()> {
        let cutoff = Utc::now() - self.retention_period;
        let mut metrics = self.metrics.lock().unwrap();

        for metric in metrics.values_mut() {
            metric.prune(cutoff);
        }

        Ok(())
    }

    /// Clear all metrics
    pub fn clear(&self) -> ColonyResult<()> {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.clear();
        Ok(())
    }

    /// Export all metrics to JSON
    pub fn export_json(&self) -> ColonyResult<String> {
        let metrics = self.list_metrics();
        serde_json::to_string_pretty(&metrics)
            .map_err(|e| crate::error::ColonyError::Colony(format!("JSON export failed: {}", e)))
    }

    /// Get statistics for a metric over a time period
    pub fn get_stats(&self, name: &str, since: Option<DateTime<Utc>>) -> Option<MetricStats> {
        let metric = self.get_metric(name)?;
        let since = since.unwrap_or_else(|| Utc::now() - Duration::hours(1));

        Some(MetricStats {
            name: metric.name.clone(),
            metric_type: metric.metric_type,
            current: metric.latest_value(),
            average: metric.average(since),
            min: metric.min(since),
            max: metric.max(since),
            sum: Some(metric.sum(since)),
            count: metric
                .points
                .iter()
                .filter(|p| p.timestamp >= since)
                .count(),
        })
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for a metric
#[derive(Debug, Clone)]
pub struct MetricStats {
    pub name: String,
    pub metric_type: MetricType,
    pub current: Option<f64>,
    pub average: Option<f64>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub sum: Option<f64>,
    pub count: usize,
}
