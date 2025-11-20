use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::ColonyResult;

/// Configuration for telemetry collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Whether telemetry is enabled (opt-in, defaults to false)
    #[serde(default)]
    pub enabled: bool,
    /// Anonymous user ID for correlation (generated on first opt-in)
    #[serde(default)]
    pub anonymous_id: Option<String>,
    /// Optional custom telemetry endpoint
    #[serde(default)]
    pub endpoint: Option<String>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            anonymous_id: None,
            endpoint: None,
        }
    }
}

impl TelemetryConfig {
    /// Get or generate anonymous ID
    pub fn get_or_create_anonymous_id(&mut self) -> String {
        if let Some(id) = &self.anonymous_id {
            id.clone()
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            self.anonymous_id = Some(id.clone());
            id
        }
    }

    /// Get the telemetry endpoint URL
    pub fn endpoint_url(&self) -> String {
        self.endpoint
            .clone()
            .unwrap_or_else(|| "https://app.colony.sh/api/telemetry".to_string())
    }
}

/// Configuration for a colony of Claude Code agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColonyConfig {
    /// Optional name for this colony (defaults to directory name)
    #[serde(default)]
    pub name: Option<String>,
    /// Optional repository configuration defining its purpose and role
    #[serde(default)]
    pub repository: Option<RepositoryConfig>,
    pub agents: Vec<AgentConfig>,
    /// Optional MCP executor configuration
    #[serde(default)]
    pub executor: Option<ExecutorConfig>,
    /// Optional shared state configuration
    #[serde(default)]
    pub shared_state: Option<crate::colony::state::SharedStateConfig>,
    /// Authentication configuration
    #[serde(default)]
    pub auth: crate::colony::auth::AuthConfig,
    /// Telemetry configuration (opt-in)
    #[serde(default)]
    pub telemetry: TelemetryConfig,
    /// Optional global capabilities available to all agents
    #[serde(default)]
    pub capabilities: Option<CapabilitiesConfig>,
    /// Optional custom layout configuration
    #[serde(default)]
    pub layout: Option<LayoutConfig>,
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
    /// Optional worktree name for sharing worktrees between agents
    /// If not specified, defaults to the agent's ID
    /// Multiple agents can specify the same worktree name to share a worktree
    #[serde(default)]
    pub worktree: Option<String>,
    /// Optional environment variables for this agent's pane
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
    /// Optional MCP servers configuration for this agent
    #[serde(default)]
    pub mcp_servers: Option<HashMap<String, McpServerConfig>>,
    /// Optional custom instructions appended to the generated prompt
    /// This will be added after the standard colony prompt (role, focus, messaging)
    #[serde(default)]
    pub instructions: Option<String>,
    #[serde(default)]
    pub capabilities: Option<CapabilitiesConfig>,
    #[serde(default)]
    pub nudge: Option<NudgeConfig>,
    /// Optional completely custom startup prompt
    /// If provided, this replaces the entire generated startup prompt
    /// Use this for complete control over the agent's initial instructions
    #[serde(default)]
    pub startup_prompt: Option<String>,
}

/// Repository type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RepositoryType {
    /// Source code repository - traditional software development
    Source,
    /// Memory/knowledge base - storing notes, research, context
    Memory,
    /// Agent application - repository IS the agent application
    Application,
    /// Research workspace - data analysis and report generation
    Research,
    /// Documentation repository - technical writing and knowledge bases
    Documentation,
}

impl Default for RepositoryType {
    fn default() -> Self {
        RepositoryType::Source
    }
}

impl RepositoryType {
    /// Get a human-readable description of this repository type
    pub fn description(&self) -> &str {
        match self {
            RepositoryType::Source => "Source code repository for software development",
            RepositoryType::Memory => "Knowledge base for storing notes, research, and context",
            RepositoryType::Application => "Agent application with state, workflows, and logic",
            RepositoryType::Research => "Research workspace for data analysis and insights",
            RepositoryType::Documentation => "Documentation repository for technical writing",
        }
    }
}

/// Configuration for repository purpose and role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    /// Type of repository (source, memory, application, research, documentation)
    #[serde(default)]
    pub repo_type: RepositoryType,
    /// Optional description of the repository's purpose
    #[serde(default)]
    pub purpose: Option<String>,
    /// Optional context about what agents should know about this repository
    #[serde(default)]
    pub context: Option<String>,
    /// Optional agent-specific capabilities
    #[serde(default)]
    pub capabilities: Option<CapabilitiesConfig>,
    /// Optional nudge configuration for this agent
    #[serde(default)]
    pub nudge: Option<NudgeConfig>,
}

/// Configuration for an MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Command to execute to start the MCP server
    pub command: String,
    /// Optional arguments to pass to the command
    #[serde(default)]
    pub args: Option<Vec<String>>,
    /// Optional environment variables for the MCP server
    #[serde(default)]
    pub env: Option<HashMap<String, String>>,
}

/// Configuration for the MCP executor pane
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorConfig {
    /// Whether the executor is enabled (defaults to false)
    #[serde(default)]
    pub enabled: bool,
    /// Agent ID for the executor (defaults to "mcp-executor")
    #[serde(default = "default_executor_id")]
    pub agent_id: String,
    /// MCP servers configuration for the executor (same format as agents)
    #[serde(default)]
    pub mcp_servers: Option<HashMap<String, McpServerConfig>>,
    /// Supported languages for MCP execution
    /// Defaults to ["typescript", "python"]
    #[serde(default = "default_languages")]
    pub languages: Vec<String>,
}

fn default_executor_id() -> String {
    "mcp-executor".to_string()
}

fn default_languages() -> Vec<String> {
    vec!["typescript".to_string(), "python".to_string()]
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            agent_id: default_executor_id(),
            mcp_servers: None,
            languages: default_languages(),
        }
    }
}

impl ExecutorConfig {
    /// Generate Claude Code settings.json content with MCP server configuration
    /// Uses the same logic as AgentConfig
    pub fn generate_settings_json(&self) -> ColonyResult<String> {
        use serde_json::{json, Value};

        let mut settings = json!({
            "bypassPermissions": true
        });

        // Add MCP servers if configured
        if let Some(mcp_servers) = &self.mcp_servers {
            let mut mcp_config: HashMap<String, Value> = HashMap::new();

            for (name, server_config) in mcp_servers {
                let mut server = json!({
                    "command": server_config.command,
                });

                // Add args if present
                if let Some(args) = &server_config.args {
                    server["args"] = json!(args);
                }

                // Add env if present
                if let Some(env) = &server_config.env {
                    server["env"] = json!(env);
                }

                mcp_config.insert(name.clone(), server);
            }

            settings["mcpServers"] = json!(mcp_config);
        }

        let json_str = serde_json::to_string_pretty(&settings)
            .map_err(|e| crate::error::ColonyError::Colony(format!("Failed to serialize settings: {}", e)))?;

        Ok(json_str)
    }

    /// Check if this executor has MCP server configuration
    pub fn has_mcp_servers(&self) -> bool {
        self.mcp_servers.as_ref().map_or(false, |s| !s.is_empty())
    }
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

    /// Get the worktree name for this agent
    /// Returns the configured worktree name, or the agent ID if not specified
    pub fn worktree_name(&self) -> &str {
        self.worktree.as_deref().unwrap_or(&self.id)
    }

    /// Generate Claude Code settings.json content with MCP server configuration
    pub fn generate_settings_json(&self) -> ColonyResult<String> {
        use serde_json::{json, Value};

        let mut settings = json!({
            "bypassPermissions": true
        });

        // Add MCP servers if configured
        if let Some(mcp_servers) = &self.mcp_servers {
            let mut mcp_config: HashMap<String, Value> = HashMap::new();

            for (name, server_config) in mcp_servers {
                let mut server = json!({
                    "command": server_config.command,
                });

                // Add args if present
                if let Some(args) = &server_config.args {
                    server["args"] = json!(args);
                }

                // Add env if present
                if let Some(env) = &server_config.env {
                    server["env"] = json!(env);
                }

                mcp_config.insert(name.clone(), server);
            }

            settings["mcpServers"] = json!(mcp_config);
        }

        let json_str = serde_json::to_string_pretty(&settings)
            .map_err(|e| crate::error::ColonyError::Colony(format!("Failed to serialize settings: {}", e)))?;

        Ok(json_str)
    }

    /// Check if this agent has MCP server configuration
    pub fn has_mcp_servers(&self) -> bool {
        self.mcp_servers.as_ref().map_or(false, |s| !s.is_empty())
    }

    /// Get resolved capabilities (agent-specific merged with global)
    pub fn resolved_capabilities(&self, global: Option<&CapabilitiesConfig>) -> Option<CapabilitiesConfig> {
        match (&self.capabilities, global) {
            (Some(agent_caps), Some(global_caps)) => Some(agent_caps.merge_with(Some(global_caps))),
            (Some(agent_caps), None) => Some(agent_caps.clone()),
            (None, Some(global_caps)) => Some(global_caps.clone()),
            (None, None) => None,
        }
    }

    /// Get nudge configuration with defaults
    pub fn nudge_config(&self) -> NudgeConfig {
        self.nudge.clone().unwrap_or_default()
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
            repository: None, // No repository config by default
            executor: None, // Executor disabled by default
            shared_state: None, // No shared state config by default
            auth: Default::default(), // Default auth (API key from env)
            telemetry: Default::default(), // Telemetry disabled by default (opt-in)
            capabilities: None,
            layout: None,
            agents: vec![
                AgentConfig {
                    id: "backend-1".to_string(),
                    role: "Backend Engineer".to_string(),
                    focus: "API endpoints and server logic".to_string(),
                    model: "claude-opus-4-20250514".to_string(),
                    directory: None,       // Uses Git worktree
                    worktree: None,        // Uses agent ID as worktree name
                    env: None,             // No custom environment variables
                    mcp_servers: None,
                    instructions: None,    // No custom instructions
                    startup_prompt: None,  // Use default generated prompt
                    capabilities: None,
                    nudge: None,
                },
                AgentConfig {
                    id: "frontend-1".to_string(),
                    role: "Frontend Engineer".to_string(),
                    focus: "React components and UI implementation".to_string(),
                    model: "claude-sonnet-4-20250514".to_string(),
                    directory: None,       // Uses Git worktree
                    worktree: None,        // Uses agent ID as worktree name
                    env: None,             // No custom environment variables
                    mcp_servers: None,
                    instructions: None,    // No custom instructions
                    startup_prompt: None,  // Use default generated prompt
                    capabilities: None,
                    nudge: None,
                },
            ],
        }
    }

    /// Get the tmux session name for this colony
    /// Uses config name if set, otherwise falls back to current directory name
    pub fn session_name(&self) -> String {
        if let Some(name) = &self.name {
            format!("colony-{}", name)
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

            format!("colony-{}", sanitized)
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

/// Nudge configuration for periodic message checking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NudgeConfig {
    /// Whether nudging is enabled for this agent
    #[serde(default = "default_nudge_enabled")]
    pub enabled: bool,
    /// Interval in seconds between nudges
    #[serde(default = "default_nudge_interval")]
    pub interval: u64,
    /// Custom nudge prompt (uses default if not specified)
    #[serde(default)]
    pub prompt: Option<String>,
}

fn default_nudge_enabled() -> bool {
    false
}

fn default_nudge_interval() -> u64 {
    60
}

impl Default for NudgeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval: 60,
            prompt: None,
        }
    }
}

/// Capabilities configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitiesConfig {
    /// Command-line tools available
    #[serde(default)]
    pub tools: Vec<String>,
    /// MCP servers available
    #[serde(default)]
    pub mcp_servers: Vec<String>,
    /// Pane-based tools (nvim, ollama, etc.)
    #[serde(default)]
    pub pane_tools: Vec<String>,
}

impl CapabilitiesConfig {
    /// Merge with global capabilities (agent overrides global)
    pub fn merge_with(&self, global: Option<&CapabilitiesConfig>) -> Self {
        let mut merged = self.clone();

        if let Some(global_caps) = global {
            // Add global tools not already in agent's list
            for tool in &global_caps.tools {
                if !merged.tools.contains(tool) {
                    merged.tools.push(tool.clone());
                }
            }
            for server in &global_caps.mcp_servers {
                if !merged.mcp_servers.contains(server) {
                    merged.mcp_servers.push(server.clone());
                }
            }
            for pane_tool in &global_caps.pane_tools {
                if !merged.pane_tools.contains(pane_tool) {
                    merged.pane_tools.push(pane_tool.clone());
                }
            }
        }

        merged
    }

    /// Export as environment variable string
    pub fn to_env_string(&self) -> String {
        format!(
            "COLONY_TOOLS=\"{}\" COLONY_MCP_SERVERS=\"{}\" COLONY_PANE_TOOLS=\"{}\"",
            self.tools.join(","),
            self.mcp_servers.join(","),
            self.pane_tools.join(",")
        )
    }
}

/// Layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    /// Layout type
    #[serde(default = "default_layout_type", rename = "type")]
    pub layout_type: String,
    /// Custom windows (optional)
    #[serde(default)]
    pub windows: Vec<WindowConfig>,
}

fn default_layout_type() -> String {
    "default".to_string()
}

/// Window configuration for custom layouts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Window name
    pub name: String,
    /// Panes in this window
    pub panes: Vec<PaneConfig>,
}

/// Pane configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneConfig {
    /// Pane type: agent, tool, executor, tui
    #[serde(rename = "type")]
    pub pane_type: String,
    /// Agent ID (for agent panes)
    #[serde(default)]
    pub agent_id: Option<String>,
    /// Command to run (for tool panes)
    #[serde(default)]
    pub command: Option<String>,
    /// Pane title
    #[serde(default)]
    pub title: Option<String>,
    /// Pane size (percentage or absolute)
    #[serde(default)]
    pub size: Option<String>,
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
            repository: None,
            shared_state: None,
            auth: Default::default(),
            telemetry: Default::default(),
            agents: vec![
                AgentConfig {
                    id: "test".to_string(),
                    role: "Test".to_string(),
                    focus: "Testing".to_string(),
                    model: "claude-sonnet-4-20250514".to_string(),
                    directory: None,
                    worktree: None,
                    env: None,
                    mcp_servers: None,
                    instructions: None,
                    startup_prompt: None,
                    capabilities: None,
                    nudge: None,
                },
                AgentConfig {
                    id: "test".to_string(),
                    role: "Test 2".to_string(),
                    focus: "Testing".to_string(),
                    model: "claude-sonnet-4-20250514".to_string(),
                    directory: None,
                    worktree: None,
                    env: None,
                    mcp_servers: None,
                    instructions: None,
                    startup_prompt: None,
                    capabilities: None,
                    nudge: None,
                },
            ],
            executor: None,
            capabilities: None,
            layout: None,
        };
        assert!(config.validate().is_err());
    }
}
