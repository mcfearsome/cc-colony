//! Configuration for git-backed state

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// State backend type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum StateBackend {
    /// Git-backed state storage
    GitBacked,
    /// In-memory only (for testing)
    Memory,
}

impl Default for StateBackend {
    fn default() -> Self {
        StateBackend::GitBacked
    }
}

/// State storage location
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum StateLocation {
    /// State stored in repository (`.colony/state/`)
    InRepo,
    /// State stored in external repository
    External,
}

impl Default for StateLocation {
    fn default() -> Self {
        StateLocation::InRepo
    }
}

/// State file schema configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSchema {
    /// Schema name (e.g., "tasks", "workflows")
    pub name: String,
    /// File path relative to state directory
    pub file: String,
    /// Whether to cache in SQLite
    #[serde(default = "default_cache")]
    pub cache: bool,
}

fn default_cache() -> bool {
    true
}

/// Shared state configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedStateConfig {
    /// Backend type
    #[serde(default)]
    pub backend: StateBackend,

    /// Storage location
    #[serde(default)]
    pub location: StateLocation,

    /// Path to state directory (relative to repo root for in-repo, absolute for external)
    #[serde(default = "default_state_path")]
    pub path: String,

    /// Path to cache database (relative to repo root)
    #[serde(default = "default_cache_path")]
    pub cache: String,

    /// Git branch to use for state
    #[serde(default = "default_branch")]
    pub branch: String,

    /// Auto-commit changes to git
    #[serde(default = "default_auto_commit")]
    pub auto_commit: bool,

    /// Auto-push commits (only if auto_commit is true)
    #[serde(default)]
    pub auto_push: bool,

    /// Commit message template
    #[serde(default = "default_commit_message")]
    pub commit_message: String,

    /// Auto-pull on colony start
    #[serde(default)]
    pub auto_pull: bool,

    /// Sync state when colony starts
    #[serde(default = "default_sync_on_start")]
    pub sync_on_start: bool,

    /// Debounce time in milliseconds before exporting
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,

    /// External repository URL (only for external location)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,

    /// Project ID within external state repo
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,

    /// State schemas
    #[serde(default = "default_schemas")]
    pub schemas: Vec<StateSchema>,
}

fn default_state_path() -> String {
    ".colony/state".to_string()
}

fn default_cache_path() -> String {
    ".colony/cache/state.db".to_string()
}

fn default_branch() -> String {
    "main".to_string()
}

fn default_auto_commit() -> bool {
    true
}

fn default_commit_message() -> String {
    "Update colony state [skip ci]".to_string()
}

fn default_sync_on_start() -> bool {
    true
}

fn default_debounce_ms() -> u64 {
    5000 // 5 seconds
}

fn default_schemas() -> Vec<StateSchema> {
    vec![
        StateSchema {
            name: "tasks".to_string(),
            file: "tasks.jsonl".to_string(),
            cache: true,
        },
        StateSchema {
            name: "workflows".to_string(),
            file: "workflows.jsonl".to_string(),
            cache: true,
        },
    ]
}

impl Default for SharedStateConfig {
    fn default() -> Self {
        Self {
            backend: StateBackend::GitBacked,
            location: StateLocation::InRepo,
            path: default_state_path(),
            cache: default_cache_path(),
            branch: default_branch(),
            auto_commit: default_auto_commit(),
            auto_push: false,
            commit_message: default_commit_message(),
            auto_pull: false,
            sync_on_start: default_sync_on_start(),
            debounce_ms: default_debounce_ms(),
            repository: None,
            project_id: None,
            schemas: default_schemas(),
        }
    }
}

impl SharedStateConfig {
    /// Get the full path to the state directory
    pub fn state_dir_path(&self, repo_root: &PathBuf) -> PathBuf {
        match self.location {
            StateLocation::InRepo => repo_root.join(&self.path),
            StateLocation::External => {
                // For external, path is absolute or relative to home
                let path = PathBuf::from(&self.path);
                if path.is_absolute() {
                    path
                } else {
                    // Expand ~ if present
                    if self.path.starts_with("~") {
                        if let Some(home) = dirs::home_dir() {
                            home.join(self.path.strip_prefix("~").unwrap())
                        } else {
                            path
                        }
                    } else {
                        path
                    }
                }
            }
        }
    }

    /// Get the full path to the cache database
    pub fn cache_db_path(&self, repo_root: &PathBuf) -> PathBuf {
        repo_root.join(&self.cache)
    }

    /// Get the schema for a given name
    pub fn get_schema(&self, name: &str) -> Option<&StateSchema> {
        self.schemas.iter().find(|s| s.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SharedStateConfig::default();
        assert_eq!(config.backend, StateBackend::GitBacked);
        assert_eq!(config.location, StateLocation::InRepo);
        assert_eq!(config.path, ".colony/state");
        assert!(config.auto_commit);
        assert!(!config.auto_push);
    }

    #[test]
    fn test_state_dir_path_in_repo() {
        let config = SharedStateConfig::default();
        let repo_root = PathBuf::from("/tmp/repo");

        let state_dir = config.state_dir_path(&repo_root);
        assert_eq!(state_dir, PathBuf::from("/tmp/repo/.colony/state"));
    }

    #[test]
    fn test_get_schema() {
        let config = SharedStateConfig::default();

        let tasks_schema = config.get_schema("tasks");
        assert!(tasks_schema.is_some());
        assert_eq!(tasks_schema.unwrap().file, "tasks.jsonl");

        let unknown_schema = config.get_schema("unknown");
        assert!(unknown_schema.is_none());
    }
}
