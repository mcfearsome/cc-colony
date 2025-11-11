use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::ColonyResult;

/// Length of shortened SHA for display purposes (standard git short SHA)
const SHORT_SHA_LENGTH: usize = 8;

/// Create a Git worktree for an agent
pub fn create_worktree(agent_id: &str, base_path: &Path) -> ColonyResult<PathBuf> {
    let worktree_path = base_path.join("worktrees").join(agent_id);

    // Ensure parent directory exists
    if let Some(parent) = worktree_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Check if worktree already exists and is valid
    if worktree_path.exists() {
        // Verify it's actually a registered git worktree
        let existing_worktrees = list_worktrees()?;
        if existing_worktrees.iter().any(|w| w == &worktree_path) {
            return Ok(worktree_path);
        }

        // Path exists but is not a valid worktree - clean it up
        crate::utils::warning(&format!(
            "Directory {} exists but is not a valid git worktree. Cleaning up...",
            worktree_path.display()
        ));
        std::fs::remove_dir_all(&worktree_path)?;
    }

    // Get current branch name or commit SHA
    let branch_output = Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .output()?;

    if !branch_output.status.success() {
        return Err(crate::error::ColonyError::Colony(format!(
            "Failed to get current branch: {}",
            String::from_utf8_lossy(&branch_output.stderr)
        )));
    }

    let branch_name = String::from_utf8_lossy(&branch_output.stdout)
        .trim()
        .to_string();

    // Handle detached HEAD state
    let ref_spec = if branch_name == "HEAD" {
        // In detached HEAD state - use the commit SHA instead
        let sha_output = Command::new("git").arg("rev-parse").arg("HEAD").output()?;

        if !sha_output.status.success() {
            return Err(crate::error::ColonyError::Colony(format!(
                "Failed to get current commit: {}",
                String::from_utf8_lossy(&sha_output.stderr)
            )));
        }

        let commit_sha = String::from_utf8_lossy(&sha_output.stdout)
            .trim()
            .to_string();

        crate::utils::warning(&format!(
            "Currently in detached HEAD state. Creating worktree from commit {}",
            &commit_sha[..SHORT_SHA_LENGTH.min(commit_sha.len())]
        ));

        commit_sha
    } else {
        branch_name
    };

    // Create worktree on the current branch or commit
    let output = Command::new("git")
        .arg("worktree")
        .arg("add")
        .arg(&worktree_path)
        .arg(&ref_spec)
        .output()?;

    if !output.status.success() {
        return Err(crate::error::ColonyError::Colony(format!(
            "Failed to create worktree: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(worktree_path)
}

/// Remove a Git worktree
pub fn remove_worktree(worktree_path: &Path) -> ColonyResult<()> {
    if !worktree_path.exists() {
        return Ok(());
    }

    let output = Command::new("git")
        .arg("worktree")
        .arg("remove")
        .arg("--force")
        .arg(worktree_path)
        .output()?;

    if !output.status.success() {
        return Err(crate::error::ColonyError::Colony(format!(
            "Failed to remove worktree: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    Ok(())
}

/// List all worktrees
pub fn list_worktrees() -> ColonyResult<Vec<PathBuf>> {
    let output = Command::new("git")
        .arg("worktree")
        .arg("list")
        .arg("--porcelain")
        .output()?;

    if !output.status.success() {
        return Err(crate::error::ColonyError::Colony(format!(
            "Failed to list worktrees: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut worktrees = Vec::new();

    for line in stdout.lines() {
        if let Some(path) = line.strip_prefix("worktree ") {
            worktrees.push(PathBuf::from(path));
        }
    }

    Ok(worktrees)
}

/// Check if we're in a Git repository
pub fn is_git_repo() -> bool {
    Command::new("git")
        .arg("rev-parse")
        .arg("--git-dir")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}
