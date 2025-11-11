use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Print a success message
pub fn success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

/// Print a warning message
pub fn warning(msg: &str) {
    println!("{} {}", "⚠".yellow().bold(), msg);
}

/// Print an info message
pub fn info(msg: &str) {
    println!("{} {}", "ℹ".blue().bold(), msg);
}

/// Print a section header
pub fn header(msg: &str) {
    println!("\n{}", msg.bold().underline());
}

/// Create a spinner progress bar
pub fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

/// Print a table row
#[allow(dead_code)]
pub fn table_row(cols: Vec<&str>, widths: Vec<usize>) {
    let row: Vec<String> = cols
        .iter()
        .zip(widths.iter())
        .map(|(col, width)| format!("{:<width$}", col, width = width))
        .collect();
    println!("{}", row.join(" │ "));
}

/// Print a table separator
#[allow(dead_code)]
pub fn table_separator(widths: Vec<usize>) {
    let sep: Vec<String> = widths.iter().map(|w| "─".repeat(*w)).collect();
    println!("{}", sep.join("─┼─"));
}

/// Confirm an action with the user
pub fn confirm(prompt: &str) -> bool {
    use dialoguer::Confirm;
    Confirm::new()
        .with_prompt(prompt)
        .default(false)
        .interact()
        .unwrap_or(false)
}

/// Select from a list of options
#[allow(dead_code)]
pub fn select(prompt: &str, items: &[String]) -> Option<usize> {
    use dialoguer::Select;
    Select::new()
        .with_prompt(prompt)
        .items(items)
        .interact()
        .ok()
}
