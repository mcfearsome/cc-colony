use std::io::{BufRead, BufReader};
use std::path::Path;

use crate::colony::logging::{LogEntry, LogFilter, LogLevel};
use crate::colony::{ColonyConfig, ColonyController};
use crate::error::ColonyResult;
use crate::utils;
use colored::Colorize;

/// View logs for an agent (legacy function for backwards compatibility)
pub async fn run(agent_id: Option<String>) -> ColonyResult<()> {
    run_with_options(agent_id, None, None, None, false, false).await
}

/// View logs for an agent with options
pub async fn run_with_options(
    agent_id: Option<String>,
    level: Option<&str>,
    pattern: Option<&str>,
    lines: Option<usize>,
    json: bool,
    no_color: bool,
) -> ColonyResult<()> {
    let config_path = Path::new("colony.yml");

    if !config_path.exists() {
        return Err(crate::error::ColonyError::Colony(
            "colony.yml not found. Run 'colony init' first.".to_string(),
        ));
    }

    // Load configuration
    let config = ColonyConfig::load(config_path)?;

    // Create controller
    let mut controller = ColonyController::new(config)?;
    controller.initialize_agents()?;

    // Build filter
    let mut filter = LogFilter::default();
    if let Some(level_str) = level {
        filter.min_level = LogLevel::from_str(level_str);
    }
    if let Some(p) = pattern {
        filter.pattern = Some(p.to_string());
    }

    // Build options
    let options = LogViewOptions {
        follow: false,
        lines,
        json,
        no_color,
        filter,
    };

    match agent_id {
        Some(id) => {
            // Show logs for specific agent
            show_agent_logs(&controller, &id, &options)?;
        }
        None => {
            // List all available logs
            list_agent_logs(&controller)?;
        }
    }

    Ok(())
}

/// Log viewing options
#[derive(Debug, Clone)]
pub struct LogViewOptions {
    pub follow: bool,
    pub lines: Option<usize>,
    pub json: bool,
    pub no_color: bool,
    pub filter: LogFilter,
}

impl Default for LogViewOptions {
    fn default() -> Self {
        Self {
            follow: false,
            lines: Some(100), // Default to last 100 lines
            json: false,
            no_color: false,
            filter: LogFilter::default(),
        }
    }
}

/// View logs for a specific agent with options
pub fn view_agent_logs(
    controller: &ColonyController,
    agent_id: &str,
    options: &LogViewOptions,
) -> ColonyResult<()> {
    let agent = controller
        .get_agent(agent_id)
        .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

    if !agent.log_path.exists() {
        utils::info("No logs available yet");
        utils::info(&format!("Log file: {}", agent.log_path.display()));
        return Ok(());
    }

    // Read log file
    let file = std::fs::File::open(&agent.log_path)?;
    let reader = BufReader::new(file);

    let mut entries: Vec<LogEntry> = Vec::new();

    // Parse log lines
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        // Try JSON first, fall back to text parsing
        let entry = LogEntry::from_json(&line)
            .or_else(|| LogEntry::from_text(&line))
            .map(|mut e| {
                // Override agent_id if not set
                if e.agent_id == "unknown" {
                    e.agent_id = agent_id.to_string();
                }
                e
            });

        if let Some(entry) = entry {
            if options.filter.matches(&entry) {
                entries.push(entry);
            }
        }
    }

    // Apply line limit
    if let Some(n) = options.lines {
        let len = entries.len();
        if len > n {
            entries = entries.into_iter().skip(len - n).collect();
        }
    }

    // Display entries
    for entry in entries {
        if options.json {
            println!("{}", entry.to_json());
        } else {
            println!("{}", entry.to_text(!options.no_color));
        }
    }

    if !options.json {
        println!();
        utils::info(&format!("Log file: {}", agent.log_path.display()));
    }

    Ok(())
}

fn show_agent_logs(
    controller: &ColonyController,
    agent_id: &str,
    options: &LogViewOptions,
) -> ColonyResult<()> {
    utils::header(&format!("Logs for agent: {}", agent_id));

    view_agent_logs(controller, agent_id, options)
}

fn list_agent_logs(controller: &ColonyController) -> ColonyResult<()> {
    utils::header("Available agent logs");

    if controller.agents().is_empty() {
        utils::info("No agents configured");
        return Ok(());
    }

    println!("{:<15} {:<10} {:<10} PATH", "AGENT ID", "SIZE", "LINES");
    println!("{}", "─".repeat(80));

    for agent in controller.agents().values() {
        let (size, lines) = if agent.log_path.exists() {
            let metadata = std::fs::metadata(&agent.log_path)?;
            let size_str = format_size(metadata.len());

            // Count lines
            let file = std::fs::File::open(&agent.log_path)?;
            let reader = BufReader::new(file);
            let line_count = reader.lines().count();

            (size_str, line_count.to_string())
        } else {
            ("-".to_string(), "-".to_string())
        };

        println!(
            "{:<15} {:<10} {:<10} {}",
            agent.id(),
            size,
            lines,
            agent.log_path.display()
        );
    }

    println!();
    utils::info("Use 'colony logs <agent-id>' to view specific logs");
    println!();
    println!("  {} View logs with filters:", "Options:".bold());
    println!("    --level <level>     Filter by log level (debug, info, warn, error)");
    println!("    --pattern <text>    Search for pattern in messages");
    println!("    --lines <n>         Show last N lines (default: 100)");
    println!("    --json              Output as JSON");
    println!("    --no-color          Disable colored output");

    Ok(())
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;

    if bytes < KB {
        format!("{} B", bytes)
    } else if bytes < MB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    }
}

/// Rotate logs for an agent (called automatically when logs get too large)
pub fn rotate_log(log_path: &Path, max_size_mb: u64) -> ColonyResult<()> {
    if !log_path.exists() {
        return Ok(());
    }

    let metadata = std::fs::metadata(log_path)?;
    let size_mb = metadata.len() / (1024 * 1024);

    if size_mb < max_size_mb {
        return Ok(());
    }

    // Rotate: log.txt -> log.txt.1, log.txt.1 -> log.txt.2, etc.
    let max_rotations = 5;

    // Delete oldest rotation
    let oldest = log_path.with_extension(format!("log.{}", max_rotations));
    if oldest.exists() {
        std::fs::remove_file(&oldest)?;
    }

    // Rotate existing files
    for i in (1..max_rotations).rev() {
        let from = log_path.with_extension(format!("log.{}", i));
        let to = log_path.with_extension(format!("log.{}", i + 1));
        if from.exists() {
            std::fs::rename(&from, &to)?;
        }
    }

    // Rotate current log
    let backup = log_path.with_extension("log.1");
    std::fs::rename(log_path, &backup)?;

    Ok(())
}
