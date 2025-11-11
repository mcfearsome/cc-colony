use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::error::ColonyResult;

/// Configuration for a colony of Claude Code agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColonyConfig {
    /// Optional name for this colony (defaults to directory name)
    #[serde(default)]
    pub name: Option<String>,
    pub agents: Vec<AgentConfig>,
}

/// Configuration for a single agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Unique identifier for this agent
    pub id: String,
    /// Human-readable role description
    pub role: String,
    /// Focus area or task description
    pub focus: String,
    /// Claude model to use (e.g., "claude-opus-4-20250514")
    #[serde(default = "default_model")]
    pub model: String,
    /// Optional custom directory/repo path for this agent
    /// If not specified, agent will work in a Git worktree of the current repo
    #[serde(default)]
    pub directory: Option<String>,
}

impl AgentConfig {
    /// Check if this agent uses a custom directory
    pub fn uses_custom_directory(&self) -> bool {
        self.directory.is_some()
    }

    /// Get the working directory for this agent
    pub fn working_directory(&self) -> Option<std::path::PathBuf> {
        self.directory.as_ref().map(|d| {
            let path = std::path::PathBuf::from(d);
            // Expand ~ to home directory (cross-platform)
            if path.starts_with("~") {
                if let Some(home) = dirs::home_dir() {
                    return home.join(path.strip_prefix("~").unwrap());
                }
            }
            path
        })
    }
}

fn default_model() -> String {
    "claude-sonnet-4-20250514".to_string()
}

impl ColonyConfig {
    /// Load colony configuration from a YAML file
    pub fn load(path: &Path) -> ColonyResult<Self> {
        let contents = fs::read_to_string(path)?;
        let config: ColonyConfig = serde_yaml::from_str(&contents)?;
        Ok(config)
    }

    /// Save colony configuration to a YAML file
    pub fn save(&self, path: &Path) -> ColonyResult<()> {
        let yaml = serde_yaml::to_string(&self)?;
        fs::write(path, yaml)?;
        Ok(())
    }

    /// Create a default colony configuration with example agents
    pub fn default() -> Self {
        ColonyConfig {
            name: None, // Will default to directory name
            agents: vec![
                AgentConfig {
                    id: "backend-1".to_string(),
                    role: "Backend Engineer".to_string(),
                    focus: "API endpoints and server logic".to_string(),
                    model: "claude-opus-4-20250514".to_string(),
                    directory: None, // Uses Git worktree
                },
                AgentConfig {
                    id: "frontend-1".to_string(),
                    role: "Frontend Engineer".to_string(),
                    focus: "React components and UI implementation".to_string(),
                    model: "claude-sonnet-4-20250514".to_string(),
                    directory: None, // Uses Git worktree
                },
            ],
        }
    }

    /// Get the tmux session name for this colony
    /// Uses config name if set, otherwise falls back to current directory name
    pub fn session_name(&self) -> String {
        if let Some(name) = &self.name {
            format!("forge-colony-{}", name)
        } else {
            // Use current directory name
            let cwd = std::env::current_dir().ok();
            let dir_name = cwd
                .as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("default");

            // Sanitize directory name for tmux (remove special chars)
            let sanitized = dir_name
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() || c == '-' || c == '_' {
                        c
                    } else {
                        '-'
                    }
                })
                .collect::<String>();

            format!("forge-colony-{}", sanitized)
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> ColonyResult<()> {
        if self.agents.is_empty() {
            return Err(crate::error::ColonyError::Colony(
                "At least one agent must be defined".to_string(),
            ));
        }

        // Check for duplicate IDs and validate ID format
        let mut ids = std::collections::HashSet::new();
        for agent in &self.agents {
            // Validate agent ID for security (prevent path traversal and shell injection)
            if !Self::is_valid_agent_id(&agent.id) {
                return Err(crate::error::ColonyError::Colony(format!(
                    "Invalid agent ID '{}'. Agent IDs must contain only alphanumeric characters, hyphens, and underscores.",
                    agent.id
                )));
            }

            if !ids.insert(&agent.id) {
                return Err(crate::error::ColonyError::Colony(format!(
                    "Duplicate agent ID: {}",
                    agent.id
                )));
            }
        }

        Ok(())
    }

    /// Check if an agent ID is valid (only alphanumeric, hyphens, and underscores)
    /// This prevents path traversal attacks and shell injection
    fn is_valid_agent_id(id: &str) -> bool {
        if id.is_empty() {
            return false;
        }

        // Only allow alphanumeric characters, hyphens, and underscores
        id.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ColonyConfig::default();
        assert_eq!(config.agents.len(), 2);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_duplicate_ids() {
        let config = ColonyConfig {
            name: None,
            agents: vec![
                AgentConfig {
                    id: "test".to_string(),
                    role: "Test".to_string(),
                    focus: "Testing".to_string(),
                    model: "claude-sonnet-4-20250514".to_string(),
                    directory: None,
                },
                AgentConfig {
                    id: "test".to_string(),
                    role: "Test 2".to_string(),
                    focus: "Testing".to_string(),
                    model: "claude-sonnet-4-20250514".to_string(),
                    directory: None,
                },
            ],
        };
        assert!(config.validate().is_err());
    }
}
