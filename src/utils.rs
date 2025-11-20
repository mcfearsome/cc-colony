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
pub fn select(prompt: &str, items: &[String]) -> Option<usize> {
    use dialoguer::Select;
    Select::new()
        .with_prompt(prompt)
        .items(items)
        .interact()
        .ok()
}

/// Prompt for text input
pub fn prompt(prompt: &str, default: Option<&str>) -> Option<String> {
    use dialoguer::Input;
    let mut input = Input::<String>::new().with_prompt(prompt);

    if let Some(d) = default {
        input = input.default(d.to_string());
    }

    input.interact_text().ok()
}

/// Prompt for number input
pub fn prompt_number(prompt: &str, default: Option<usize>) -> Option<usize> {
    use dialoguer::Input;
    let mut input = Input::<usize>::new().with_prompt(prompt);

    if let Some(d) = default {
        input = input.default(d);
    }

    input.interact_text().ok()
}

/// Multi-select from a list of options
pub fn multiselect(prompt: &str, items: &[String], defaults: &[bool]) -> Option<Vec<usize>> {
    use dialoguer::MultiSelect;
    let mut ms = MultiSelect::new().with_prompt(prompt).items(items);

    // Set defaults if provided
    if !defaults.is_empty() {
        ms = ms.defaults(defaults);
    }

    ms.interact().ok()
}
