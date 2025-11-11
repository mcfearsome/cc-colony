use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Print a success message
pub fn success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

/// Print an error message
pub fn error(msg: &str) {
    eprintln!("{} {}", "✗".red().bold(), msg);
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

/// Create a progress bar with known length
pub fn progress_bar(len: u64, msg: &str) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message(msg.to_string());
    pb
}

/// Format a server name with color
pub fn format_server_name(name: &str) -> String {
    name.cyan().bold().to_string()
}

/// Format a version string
pub fn format_version(version: &str) -> String {
    version.green().to_string()
}

/// Format a tag
pub fn format_tag(tag: &str) -> String {
    format!("[{}]", tag.yellow())
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

/// Prompt for text input
pub fn input(prompt: &str) -> Option<String> {
    use dialoguer::Input;
    Input::new().with_prompt(prompt).interact_text().ok()
}

/// Prompt for password
pub fn password(prompt: &str) -> Option<String> {
    use dialoguer::Password;
    Password::new().with_prompt(prompt).interact().ok()
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
