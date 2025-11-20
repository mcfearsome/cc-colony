use crate::colony::metrics::{standard_metrics, MetricType, MetricsCollector};
use crate::error::ColonyResult;
use crate::utils;
use chrono::{Duration, Utc};
use colored::Colorize;
use std::collections::HashMap;

/// List all registered metrics
pub fn list_metrics() -> ColonyResult<()> {
    let collector = get_or_init_collector();
    let metrics = collector.list_metrics();

    if metrics.is_empty() {
        println!("{}", "ℹ No metrics available yet.".dimmed());
        println!();
        println!("Metrics are collected automatically when agents are running.");
        println!("Start a colony with 'colony start' to begin collecting metrics.");
        return Ok(());
    }

    println!("{}", "Available Metrics".bold());
    println!();

    // Group by category
    let mut agent_metrics = Vec::new();
    let mut task_metrics = Vec::new();
    let mut workflow_metrics = Vec::new();
    let mut system_metrics = Vec::new();

    for metric in metrics {
        if metric.name.starts_with("agent.") {
            agent_metrics.push(metric);
        } else if metric.name.starts_with("task.") {
            task_metrics.push(metric);
        } else if metric.name.starts_with("workflow.") {
            workflow_metrics.push(metric);
        } else if metric.name.starts_with("system.") {
            system_metrics.push(metric);
        }
    }

    if !agent_metrics.is_empty() {
        println!("{}", "Agent Metrics".green().bold());
        for metric in agent_metrics {
            print_metric_summary(&metric);
        }
        println!();
    }

    if !task_metrics.is_empty() {
        println!("{}", "Task Metrics".blue().bold());
        for metric in task_metrics {
            print_metric_summary(&metric);
        }
        println!();
    }

    if !workflow_metrics.is_empty() {
        println!("{}", "Workflow Metrics".yellow().bold());
        for metric in workflow_metrics {
            print_metric_summary(&metric);
        }
        println!();
    }

    if !system_metrics.is_empty() {
        println!("{}", "System Metrics".magenta().bold());
        for metric in system_metrics {
            print_metric_summary(&metric);
        }
        println!();
    }

    println!(
        "Use {} to see detailed statistics",
        "'colony metrics show <name>'".dimmed()
    );

    Ok(())
}

fn print_metric_summary(metric: &crate::colony::metrics::Metric) {
    let type_str = match metric.metric_type {
        MetricType::Counter => "counter".cyan(),
        MetricType::Gauge => "gauge".yellow(),
        MetricType::Histogram => "histogram".magenta(),
    };

    let value_str = if let Some(value) = metric.latest_value() {
        if let Some(ref unit) = metric.unit {
            format!("{:.2} {}", value, unit)
        } else {
            format!("{:.2}", value)
        }
    } else {
        "no data".dimmed().to_string()
    };

    println!(
        "  {} {} - {}",
        metric.name.bold(),
        format!("[{}]", type_str).dimmed(),
        value_str
    );
    println!("    {}", metric.description.dimmed());
}

/// Show detailed statistics for a specific metric
pub fn show_metric(name: &str, hours: Option<usize>) -> ColonyResult<()> {
    let collector = get_or_init_collector();
    let metric = collector
        .get_metric(name)
        .ok_or_else(|| crate::error::ColonyError::Colony(format!("Metric not found: {}", name)))?;

    let hours = hours.unwrap_or(1);
    let since = Utc::now() - Duration::hours(hours as i64);

    println!("{}: {}", "Metric".bold(), metric.name.cyan());
    println!("{}: {}", "Type".bold(), format!("{:?}", metric.metric_type));
    println!("{}: {}", "Description".bold(), metric.description);
    if let Some(ref unit) = metric.unit {
        println!("{}: {}", "Unit".bold(), unit);
    }
    println!();

    let stats = collector.get_stats(name, Some(since)).unwrap();

    println!("{} (last {} hours)", "Statistics".bold(), hours);
    println!();

    if let Some(current) = stats.current {
        println!(
            "  {}: {}",
            "Current".bold(),
            format_value(current, &metric.unit)
        );
    }

    if let Some(avg) = stats.average {
        println!(
            "  {}: {}",
            "Average".bold(),
            format_value(avg, &metric.unit)
        );
    }

    if let Some(min) = stats.min {
        println!("  {}: {}", "Min".bold(), format_value(min, &metric.unit));
    }

    if let Some(max) = stats.max {
        println!("  {}: {}", "Max".bold(), format_value(max, &metric.unit));
    }

    if let Some(sum) = stats.sum {
        if metric.metric_type == MetricType::Counter {
            println!("  {}: {}", "Total".bold(), format_value(sum, &metric.unit));
        }
    }

    println!("  {}: {}", "Data Points".bold(), stats.count);
    println!();

    // Show recent values
    let recent: Vec<_> = metric
        .points
        .iter()
        .filter(|p| p.timestamp >= since)
        .rev()
        .take(10)
        .collect();

    if !recent.is_empty() {
        println!("{}", "Recent Values".bold());
        println!();
        for point in recent {
            let timestamp = point.timestamp.format("%Y-%m-%d %H:%M:%S");
            let value = format_value(point.value, &metric.unit);

            if point.labels.is_empty() {
                println!("  {} - {}", timestamp.to_string().dimmed(), value);
            } else {
                let labels: Vec<String> = point
                    .labels
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect();
                println!(
                    "  {} - {} {}",
                    timestamp.to_string().dimmed(),
                    value,
                    format!("[{}]", labels.join(", ")).dimmed()
                );
            }
        }
    }

    Ok(())
}

/// Export metrics to JSON
pub fn export_metrics(output: Option<&str>) -> ColonyResult<()> {
    let collector = get_or_init_collector();
    let json = collector.export_json()?;

    if let Some(path) = output {
        std::fs::write(path, json)
            .map_err(|e| crate::error::ColonyError::Colony(format!("Write failed: {}", e)))?;
        utils::success(&format!("Metrics exported to: {}", path));
    } else {
        println!("{}", json);
    }

    Ok(())
}

/// Clear old metrics data
pub fn clear_metrics(all: bool) -> ColonyResult<()> {
    let collector = get_or_init_collector();

    if all {
        collector.clear()?;
        utils::success("All metrics cleared");
    } else {
        collector.prune_old_data()?;
        utils::success("Old metrics data pruned");
    }

    Ok(())
}

/// Initialize sample metrics for demonstration
pub fn init_sample_metrics() -> ColonyResult<()> {
    let collector = get_or_init_collector();

    // Register standard metrics
    collector.register_metric(
        standard_metrics::AGENT_TASKS_COMPLETED.to_string(),
        MetricType::Counter,
        "Number of tasks completed by agents".to_string(),
        Some("tasks".to_string()),
    )?;

    collector.register_metric(
        standard_metrics::AGENT_TASKS_FAILED.to_string(),
        MetricType::Counter,
        "Number of tasks failed by agents".to_string(),
        Some("tasks".to_string()),
    )?;

    collector.register_metric(
        standard_metrics::TASK_QUEUE_DEPTH.to_string(),
        MetricType::Gauge,
        "Current number of tasks in the queue".to_string(),
        Some("tasks".to_string()),
    )?;

    collector.register_metric(
        standard_metrics::TASK_EXECUTION_TIME.to_string(),
        MetricType::Histogram,
        "Time taken to execute tasks".to_string(),
        Some("seconds".to_string()),
    )?;

    collector.register_metric(
        standard_metrics::WORKFLOW_RUNS_COMPLETED.to_string(),
        MetricType::Counter,
        "Number of workflow runs completed".to_string(),
        Some("runs".to_string()),
    )?;

    collector.register_metric(
        standard_metrics::SYSTEM_MEMORY_USED.to_string(),
        MetricType::Gauge,
        "Memory used by the colony system".to_string(),
        Some("MB".to_string()),
    )?;

    utils::success("Sample metrics initialized");
    Ok(())
}

/// Record a sample metric (for testing)
pub fn record_sample(name: &str, value: f64) -> ColonyResult<()> {
    let collector = get_or_init_collector();
    collector.record_simple(name, value)?;
    utils::success(&format!("Recorded {} = {}", name, value));
    Ok(())
}

fn format_value(value: f64, unit: &Option<String>) -> String {
    if let Some(u) = unit {
        format!("{:.2} {}", value, u)
    } else {
        format!("{:.2}", value)
    }
}

// Global metrics collector (in production, this would be passed around or in a context)
fn get_or_init_collector() -> MetricsCollector {
    // For now, create a new collector each time
    // In production, this should be a singleton or passed through the app context
    let collector = MetricsCollector::new();

    // Auto-register standard metrics
    let _ = collector.register_metric(
        standard_metrics::AGENT_TASKS_COMPLETED.to_string(),
        MetricType::Counter,
        "Number of tasks completed by agents".to_string(),
        Some("tasks".to_string()),
    );

    let _ = collector.register_metric(
        standard_metrics::AGENT_TASKS_FAILED.to_string(),
        MetricType::Counter,
        "Number of tasks failed by agents".to_string(),
        Some("tasks".to_string()),
    );

    let _ = collector.register_metric(
        standard_metrics::TASK_QUEUE_DEPTH.to_string(),
        MetricType::Gauge,
        "Current number of tasks in the queue".to_string(),
        Some("tasks".to_string()),
    );

    let _ = collector.register_metric(
        standard_metrics::TASK_EXECUTION_TIME.to_string(),
        MetricType::Histogram,
        "Time taken to execute tasks".to_string(),
        Some("seconds".to_string()),
    );

    let _ = collector.register_metric(
        standard_metrics::WORKFLOW_RUNS_STARTED.to_string(),
        MetricType::Counter,
        "Number of workflow runs started".to_string(),
        Some("runs".to_string()),
    );

    let _ = collector.register_metric(
        standard_metrics::WORKFLOW_RUNS_COMPLETED.to_string(),
        MetricType::Counter,
        "Number of workflow runs completed".to_string(),
        Some("runs".to_string()),
    );

    let _ = collector.register_metric(
        standard_metrics::SYSTEM_MEMORY_USED.to_string(),
        MetricType::Gauge,
        "Memory used by the colony system".to_string(),
        Some("MB".to_string()),
    );

    collector
}
