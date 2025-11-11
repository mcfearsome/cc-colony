use crate::error::ForgeResult;
use std::process::Command;

/// Check if tmux is installed and available
pub fn is_tmux_available() -> bool {
    Command::new("tmux")
        .arg("-V")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Attempt to install tmux automatically
pub fn try_install_tmux() -> ForgeResult<()> {
    // Detect OS and try appropriate package manager
    if cfg!(target_os = "macos") {
        // Try homebrew
        let output = Command::new("brew").arg("install").arg("tmux").output()?;

        if !output.status.success() {
            return Err(crate::error::ForgeError::Swarm(
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
                    return Err(crate::error::ForgeError::Swarm(
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
            return Err(crate::error::ForgeError::Swarm(
                "Could not find a supported package manager to install tmux.".to_string(),
            ));
        }
    } else {
        return Err(crate::error::ForgeError::Swarm(
            "Automatic tmux installation not supported on this platform.".to_string(),
        ));
    }

    // Verify installation
    if !is_tmux_available() {
        return Err(crate::error::ForgeError::Swarm(
            "tmux installation appeared to succeed but tmux is still not available.".to_string(),
        ));
    }

    Ok(())
}

/// Check if a tmux session exists
pub fn session_exists(session_name: &str) -> bool {
    Command::new("tmux")
        .arg("has-session")
        .arg("-t")
        .arg(session_name)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Create a new tmux session
pub fn create_session(session_name: &str) -> ForgeResult<()> {
    let output = Command::new("tmux")
        .arg("new-session")
        .arg("-d")
        .arg("-s")
        .arg(session_name)
        .output()?;

    if !output.status.success() {
        return Err(crate::error::ForgeError::Swarm(format!(
            "Failed to create tmux session: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Kill a tmux session
pub fn kill_session(session_name: &str) -> ForgeResult<()> {
    if !session_exists(session_name) {
        return Ok(());
    }

    let output = Command::new("tmux")
        .arg("kill-session")
        .arg("-t")
        .arg(session_name)
        .output()?;

    if !output.status.success() {
        return Err(crate::error::ForgeError::Swarm(format!(
            "Failed to kill tmux session: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Split a tmux window horizontally and run a command in the new pane
pub fn split_horizontal(session_name: &str, command: &str) -> ForgeResult<()> {
    let output = Command::new("tmux")
        .arg("split-window")
        .arg("-h")
        .arg("-t")
        .arg(session_name)
        .arg(command)
        .output()?;

    if !output.status.success() {
        return Err(crate::error::ForgeError::Swarm(format!(
            "Failed to split tmux pane: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Split a tmux window vertically and run a command in the new pane
pub fn split_vertical(session_name: &str, command: &str) -> ForgeResult<()> {
    let output = Command::new("tmux")
        .arg("split-window")
        .arg("-v")
        .arg("-t")
        .arg(session_name)
        .arg(command)
        .output()?;

    if !output.status.success() {
        return Err(crate::error::ForgeError::Swarm(format!(
            "Failed to split tmux pane: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Run a command in the first pane of a session
pub fn send_command_to_pane(session_name: &str, pane: usize, command: &str) -> ForgeResult<()> {
    let target = format!("{}:{}", session_name, pane);

    let output = Command::new("tmux")
        .arg("send-keys")
        .arg("-t")
        .arg(&target)
        .arg(command)
        .arg("C-m")
        .output()?;

    if !output.status.success() {
        return Err(crate::error::ForgeError::Swarm(format!(
            "Failed to send command to tmux pane: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Attach to a tmux session
pub fn attach_session(session_name: &str) -> ForgeResult<()> {
    let output = Command::new("tmux")
        .arg("attach-session")
        .arg("-t")
        .arg(session_name)
        .status()?;

    if !output.success() {
        return Err(crate::error::ForgeError::Swarm(
            "Failed to attach to tmux session".to_string(),
        ));
    }

    Ok(())
}

/// Select a tiled layout for the tmux window
pub fn select_tiled_layout(session_name: &str) -> ForgeResult<()> {
    let output = Command::new("tmux")
        .arg("select-layout")
        .arg("-t")
        .arg(session_name)
        .arg("tiled")
        .output()?;

    if !output.status.success() {
        return Err(crate::error::ForgeError::Swarm(format!(
            "Failed to set tmux layout: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// Set pane title
pub fn set_pane_title(session_name: &str, pane: usize, title: &str) -> ForgeResult<()> {
    let target = format!("{}:{}", session_name, pane);

    let output = Command::new("tmux")
        .arg("select-pane")
        .arg("-t")
        .arg(&target)
        .arg("-T")
        .arg(title)
        .output()?;

    if !output.status.success() {
        return Err(crate::error::ForgeError::Swarm(format!(
            "Failed to set pane title: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}
