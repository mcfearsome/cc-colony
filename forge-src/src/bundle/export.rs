use crate::config::ClaudeConfig;
use crate::error::ForgeResult;
use crate::utils;
use chrono::Utc;
use std::fs;

pub async fn run(output: Option<String>) -> ForgeResult<()> {
    let config = ClaudeConfig::load()?;

    // Generate default filename if not provided
    let filename = output.unwrap_or_else(|| {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        format!("claude_config_backup_{}.json", timestamp)
    });

    utils::header(&format!("Exporting configuration to '{}'", filename));

    let spinner = utils::spinner("Exporting...");

    let content = serde_json::to_string_pretty(&config)?;
    fs::write(&filename, content)?;

    spinner.finish_and_clear();

    utils::success(&format!("Configuration exported to '{}'", filename));
    utils::info(&format!(
        "File size: {} bytes",
        fs::metadata(&filename)?.len()
    ));

    Ok(())
}
