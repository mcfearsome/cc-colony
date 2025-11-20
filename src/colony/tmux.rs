use crate::error::ColonyResult;
use std::process::Command;

/// Get the tmux binary path
/// Tries common locations and falls back to PATH
fn tmux_bin() -> &'static str {
    // Check common locations first (Homebrew on different architectures)
    if std::path::Path::new("/opt/homebrew/bin/tmux").exists() {
        "/opt/homebrew/bin/tmux"
    } else if std::path::Path::new("/usr/local/bin/tmux").exists() {
        "/usr/local/bin/tmux"
    } else {
        // Fall back to PATH
        "tmux"
    }
}

/// Check if tmux is installed and available
pub fn is_tmux_available() -> bool {
    Command::new(tmux_bin())
        .arg("-V")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Attempt to install tmux automatically
pub fn try_install_tmux() -> ColonyResult<()> {
    // Detect OS and try appropriate package manager
    if cfg!(target_os = "macos") {
        // Try homebrew
        let output = Command::new("brew").arg("install").arg("tmux").output()?;

        if !output.status.success() {
            return Err(crate::error::ColonyError::Colony(
                "Failed to install tmux via homebrew. Please install manually.".to_string(),
            ));
        }
    } else if cfg!(target_os = "linux") {
        // Try common Linux package managers in order
        let managers = vec![
            ("apt-get", vec!["install", "-y", "tmux"]),
            ("dnf", vec!["install", "-y", "tmux"]),
            ("yum", vec!["install", "-y", "tmux"]),
            ("pacman", vec!["-S", "--noconfirm", "tmux"]),
            ("zypper", vec!["install", "-y", "tmux"]),
        ];

        let mut installed = false;
        for (manager, args) in managers {
            // Check if package manager exists
            if Command::new("which")
                .arg(manager)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                // Ask for user confirmation before running sudo
                crate::utils::warning(&format!(
                    "tmux installation requires sudo access to run: sudo {} {}",
                    manager,
                    args.join(" ")
                ));

                if !crate::utils::confirm("Do you want to proceed with tmux installation?") {
                    return Err(crate::error::ColonyError::Colony(
                        "tmux installation cancelled by user".to_string(),
                    ));
                }

                // Try to install with sudo
                let output = Command::new("sudo").arg(manager).args(&args).output()?;

                if output.status.success() {
                    installed = true;
                    break;
                }
            }
        }

        if !installed {
            return Err(crate::error::ColonyError::Colony(
                "Could not find a supported package manager to install tmux.".to_string(),
            ));
        }
    } else {
        return Err(crate::error::ColonyError::Colony(
            "Automatic tmux installation not supported on this platform.".to_string(),
        ));
    }

    // Verify installation
    if !is_tmux_available() {
        return Err(crate::error::ColonyError::Colony(
            "tmux installation appeared to succeed but tmux is still not available.".to_string(),
        ));
    }

    Ok(())
}

/// Check if a tmux session exists
pub fn session_exists(session_name: &str) -> bool {
    Command::new(tmux_bin())
        .arg("has-session")
        .arg("-t")
        .arg(session_name)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Create a new tmux session
pub fn create_session(session_name: &str) -> ColonyResult<()> {
    let output = Command::new(tmux_bin())
        .arg("new-session")
        .arg("-d")
        .arg("-s")
        .arg(session_name)
        .output()?;

    if !output.status.success() {
        return Err(crate::error::ColonyError::Colony(format!(
            "Failed to create tmux session: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Kill a tmux session
pub fn kill_session(session_name: &str) -> ColonyResult<()> {
    if !session_exists(session_name) {
        return Ok(());
    }

    let output = Command::new(tmux_bin())
        .arg("kill-session")
        .arg("-t")
        .arg(session_name)
        .output()?;

    if !output.status.success() {
        return Err(crate::error::ColonyError::Colony(format!(
            "Failed to kill tmux session: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Split a tmux window horizontally and run a command in the new pane
/// Returns the index of the newly created pane
pub fn split_horizontal(session_name: &str, command: &str) -> ColonyResult<usize> {
    // Use full path to tmux and wrap command in sh -c to ensure proper shell execution
    let output = Command::new(tmux_bin())
        .arg("split-window")
        .arg("-h")
        .arg("-t")
        .arg(session_name)
        .arg("-P") // Print info about new pane
        .arg("-F")
        .arg("#{pane_index}") // Format: just print the pane index
        .arg("sh")
        .arg("-c")
        .arg(command)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(crate::error::ColonyError::Colony(format!(
            "Failed to split tmux pane horizontally:\nstderr: {}\nstdout: {}",
            stderr, stdout
        )));
    }

    // Parse pane index from output
    let pane_index_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let pane_index = pane_index_str.parse::<usize>().map_err(|_| {
        crate::error::ColonyError::Colony(format!(
            "Failed to parse pane index: '{}' (length: {})",
            pane_index_str,
            pane_index_str.len()
        ))
    })?;

    Ok(pane_index)
}

/// Split a tmux window vertically and run a command in the new pane
/// Returns the index of the newly created pane
pub fn split_vertical(session_name: &str, command: &str) -> ColonyResult<usize> {
    // Use full path to tmux and wrap command in sh -c to ensure proper shell execution
    let output = Command::new(tmux_bin())
        .arg("split-window")
        .arg("-v")
        .arg("-t")
        .arg(session_name)
        .arg("-P") // Print info about new pane
        .arg("-F")
        .arg("#{pane_index}") // Format: just print the pane index
        .arg("sh")
        .arg("-c")
        .arg(command)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(crate::error::ColonyError::Colony(format!(
            "Failed to split tmux pane vertically:\nstderr: {}\nstdout: {}",
            stderr, stdout
        )));
    }

    // Parse pane index from output
    let pane_index_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let pane_index = pane_index_str.parse::<usize>().map_err(|_| {
        crate::error::ColonyError::Colony(format!(
            "Failed to parse pane index: '{}' (length: {})",
            pane_index_str,
            pane_index_str.len()
        ))
    })?;

    Ok(pane_index)
}

/// Run a command in the first pane of a session
pub fn send_command_to_pane(session_name: &str, pane: usize, command: &str) -> ColonyResult<()> {
    // Use window.pane format (default window is 0)
    let target = format!("{}:0.{}", session_name, pane);

    let output = Command::new(tmux_bin())
        .arg("send-keys")
        .arg("-t")
        .arg(&target)
        .arg(command)
        .arg("C-m")
        .output()?;

    if !output.status.success() {
        return Err(crate::error::ColonyError::Colony(format!(
            "Failed to send command to tmux pane: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Run a command in a specific window:pane (for multi-window sessions)
pub fn send_command_to_window_pane(
    session_name: &str,
    window: usize,
    pane: usize,
    command: &str,
) -> ColonyResult<()> {
    let target = format!("{}:{}.{}", session_name, window, pane);

    let output = Command::new(tmux_bin())
        .arg("send-keys")
        .arg("-t")
        .arg(&target)
        .arg(command)
        .arg("C-m")
        .output()?;

    if !output.status.success() {
        return Err(crate::error::ColonyError::Colony(format!(
            "Failed to send command to pane {}: {}",
            target,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Set pane title for a specific window:pane (for multi-window sessions)
pub fn set_window_pane_title(
    session_name: &str,
    window: usize,
    pane: usize,
    title: &str,
) -> ColonyResult<()> {
    let target = format!("{}:{}.{}", session_name, window, pane);

    let output = Command::new(tmux_bin())
        .arg("select-pane")
        .arg("-t")
        .arg(&target)
        .arg("-T")
        .arg(title)
        .output()?;

    if !output.status.success() {
        return Err(crate::error::ColonyError::Colony(format!(
            "Failed to set pane title for {}: {}",
            target,
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Attach to a tmux session
pub fn attach_session(session_name: &str) -> ColonyResult<()> {
    let output = Command::new(tmux_bin())
        .arg("attach-session")
        .arg("-t")
        .arg(session_name)
        .status()?;

    if !output.success() {
        return Err(crate::error::ColonyError::Colony(
            "Failed to attach to tmux session".to_string(),
        ));
    }

    Ok(())
}

/// Select a tiled layout for the tmux window
pub fn select_tiled_layout(session_name: &str) -> ColonyResult<()> {
    let output = Command::new(tmux_bin())
        .arg("select-layout")
        .arg("-t")
        .arg(session_name)
        .arg("tiled")
        .output()?;

    if !output.status.success() {
        return Err(crate::error::ColonyError::Colony(format!(
            "Failed to set tmux layout: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Select a specific layout for a window
pub fn select_window_layout(session_name: &str, window: usize, layout: &str) -> ColonyResult<()> {
    let target = format!("{}:{}", session_name, window);

    let output = Command::new(tmux_bin())
        .arg("select-layout")
        .arg("-t")
        .arg(&target)
        .arg(layout)
        .output()?;

    if !output.status.success() {
        return Err(crate::error::ColonyError::Colony(format!(
            "Failed to set window layout: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Resize a pane by percentage
pub fn resize_pane_percentage(
    session_name: &str,
    window: usize,
    pane: usize,
    width_pct: Option<u16>,
    height_pct: Option<u16>,
) -> ColonyResult<()> {
    let target = format!("{}:{}.{}", session_name, window, pane);

    if let Some(width) = width_pct {
        let output = Command::new(tmux_bin())
            .arg("resize-pane")
            .arg("-t")
            .arg(&target)
            .arg("-x")
            .arg(&format!("{}%", width))
            .output()?;

        if !output.status.success() {
            return Err(crate::error::ColonyError::Colony(format!(
                "Failed to resize pane width: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
    }

    if let Some(height) = height_pct {
        let output = Command::new(tmux_bin())
            .arg("resize-pane")
            .arg("-t")
            .arg(&target)
            .arg("-y")
            .arg(&format!("{}%", height))
            .output()?;

        if !output.status.success() {
            return Err(crate::error::ColonyError::Colony(format!(
                "Failed to resize pane height: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }
    }

    Ok(())
}

/// Swap two panes
pub fn swap_panes(
    session_name: &str,
    src_window: usize,
    src_pane: usize,
    dst_window: usize,
    dst_pane: usize,
) -> ColonyResult<()> {
    let src_target = format!("{}:{}.{}", session_name, src_window, src_pane);
    let dst_target = format!("{}:{}.{}", session_name, dst_window, dst_pane);

    let output = Command::new(tmux_bin())
        .arg("swap-pane")
        .arg("-s")
        .arg(&src_target)
        .arg("-t")
        .arg(&dst_target)
        .output()?;

    if !output.status.success() {
        return Err(crate::error::ColonyError::Colony(format!(
            "Failed to swap panes: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Apply a custom layout string to a window
/// Layout strings define exact pane geometry (get with: tmux list-windows -F "#{window_layout}")
pub fn apply_custom_layout(
    session_name: &str,
    window: usize,
    layout_string: &str,
) -> ColonyResult<()> {
    let target = format!("{}:{}", session_name, window);

    let output = Command::new(tmux_bin())
        .arg("select-layout")
        .arg("-t")
        .arg(&target)
        .arg(layout_string)
        .output()?;

    if !output.status.success() {
        return Err(crate::error::ColonyError::Colony(format!(
            "Failed to apply custom layout: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Set pane title
pub fn set_pane_title(session_name: &str, pane: usize, title: &str) -> ColonyResult<()> {
    // Use window.pane format (default window is 0)
    // Format: session:window.pane (e.g., colony-gusto-web:0.1 for pane 1 in window 0)
    let target = format!("{}:0.{}", session_name, pane);

    let output = Command::new(tmux_bin())
        .arg("select-pane")
        .arg("-t")
        .arg(&target)
        .arg("-T")
        .arg(title)
        .output()?;

    if !output.status.success() {
        return Err(crate::error::ColonyError::Colony(format!(
            "Failed to set pane title: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Break a pane out into a new window and return the new window index
pub fn break_pane(session_name: &str, window: usize, pane: usize) -> ColonyResult<usize> {
    let target = format!("{}:{}.{}", session_name, window, pane);

    // First select the pane (break-pane operates on selected pane)
    let _select = Command::new(tmux_bin())
        .arg("select-pane")
        .arg("-t")
        .arg(&target)
        .output()?;

    // Now break the selected pane
    let output = Command::new(tmux_bin())
        .arg("break-pane")
        .arg("-d") // Don't switch to new window
        .arg("-P")
        .arg("-F")
        .arg("#{window_index}")
        .output()?;

    if !output.status.success() {
        return Err(crate::error::ColonyError::Colony(format!(
            "Failed to break pane: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let window_idx_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let window_idx = window_idx_str.parse::<usize>().map_err(|_| {
        crate::error::ColonyError::Colony(format!(
            "Failed to parse window index: {}",
            window_idx_str
        ))
    })?;

    Ok(window_idx)
}

/// Join a pane from one window into another window at a specific position
pub fn join_pane_at(
    session_name: &str,
    src_window: usize,
    dst_window: usize,
    dst_pane: usize,
    vertical: bool,
    before: bool,
) -> ColonyResult<()> {
    let src_target = format!("{}:{}", session_name, src_window);
    let dst_target = format!("{}:{}.{}", session_name, dst_window, dst_pane);

    let mut cmd = Command::new(tmux_bin());
    cmd.arg("join-pane")
        .arg("-s")
        .arg(&src_target)
        .arg("-t")
        .arg(&dst_target);

    if vertical {
        cmd.arg("-v"); // Join vertically (above/below)
    } else {
        cmd.arg("-h"); // Join horizontally (left/right)
    }

    if before {
        cmd.arg("-b"); // Join before target instead of after
    }

    let output = cmd.output()?;

    if !output.status.success() {
        return Err(crate::error::ColonyError::Colony(format!(
            "Failed to join pane: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}
