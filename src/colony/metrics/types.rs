use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of metric
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    /// Monotonically increasing counter
    Counter,
    /// Point-in-time value that can go up or down
    Gauge,
    /// Distribution of values over time
    Histogram,
}

/// A single metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub labels: HashMap<String, String>,
}

/// A metric with its metadata and data points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub metric_type: MetricType,
    pub description: String,
    pub unit: Option<String>,
    pub points: Vec<MetricPoint>,
}

impl Metric {
    pub fn new(name: String, metric_type: MetricType, description: String) -> Self {
        Self {
            name,
            metric_type,
            description,
            unit: None,
            points: Vec::new(),
        }
    }

    pub fn with_unit(mut self, unit: String) -> Self {
        self.unit = Some(unit);
        self
    }

    /// Add a data point
    pub fn record(&mut self, value: f64, labels: HashMap<String, String>) {
        self.points.push(MetricPoint {
            timestamp: Utc::now(),
            value,
            labels,
        });
    }

    /// Get the latest value
    pub fn latest_value(&self) -> Option<f64> {
        self.points.last().map(|p| p.value)
    }

    /// Get average value over time period
    pub fn average(&self, since: DateTime<Utc>) -> Option<f64> {
        let points: Vec<_> = self
            .points
            .iter()
            .filter(|p| p.timestamp >= since)
            .collect();

        if points.is_empty() {
            return None;
        }

        let sum: f64 = points.iter().map(|p| p.value).sum();
        Some(sum / points.len() as f64)
    }

    /// Get sum of values (useful for counters)
    pub fn sum(&self, since: DateTime<Utc>) -> f64 {
        self.points
            .iter()
            .filter(|p| p.timestamp >= since)
            .map(|p| p.value)
            .sum()
    }

    /// Get max value over time period
    pub fn max(&self, since: DateTime<Utc>) -> Option<f64> {
        self.points
            .iter()
            .filter(|p| p.timestamp >= since)
            .map(|p| p.value)
            .fold(None, |max, v| match max {
                None => Some(v),
                Some(m) => Some(if v > m { v } else { m }),
            })
    }

    /// Get min value over time period
    pub fn min(&self, since: DateTime<Utc>) -> Option<f64> {
        self.points
            .iter()
            .filter(|p| p.timestamp >= since)
            .map(|p| p.value)
            .fold(None, |min, v| match min {
                None => Some(v),
                Some(m) => Some(if v < m { v } else { m }),
            })
    }

    /// Remove old data points
    pub fn prune(&mut self, before: DateTime<Utc>) {
        self.points.retain(|p| p.timestamp >= before);
    }
}

/// Standard metric names used by Colony
pub mod standard_metrics {
    pub const AGENT_TASKS_COMPLETED: &str = "agent.tasks.completed";
    pub const AGENT_TASKS_FAILED: &str = "agent.tasks.failed";
    pub const AGENT_ACTIVE_TIME: &str = "agent.active_time";
    pub const AGENT_IDLE_TIME: &str = "agent.idle_time";

    pub const TASK_QUEUE_DEPTH: &str = "task.queue.depth";
    pub const TASK_WAIT_TIME: &str = "task.wait_time";
    pub const TASK_EXECUTION_TIME: &str = "task.execution_time";

    pub const WORKFLOW_RUNS_STARTED: &str = "workflow.runs.started";
    pub const WORKFLOW_RUNS_COMPLETED: &str = "workflow.runs.completed";
    pub const WORKFLOW_RUNS_FAILED: &str = "workflow.runs.failed";
    pub const WORKFLOW_STEP_DURATION: &str = "workflow.step.duration";

    pub const SYSTEM_MEMORY_USED: &str = "system.memory.used";
    pub const SYSTEM_CPU_PERCENT: &str = "system.cpu.percent";
}
