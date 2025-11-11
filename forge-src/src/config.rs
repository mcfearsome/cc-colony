use crate::error::{ForgeError, ForgeResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ClaudeConfig {
    #[serde(default)]
    pub mcpServers: HashMap<String, McpServer>,

    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub command: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,

    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

impl ClaudeConfig {
    /// Create a new empty config
    pub fn new() -> Self {
        Self {
            mcpServers: HashMap::new(),
            other: HashMap::new(),
        }
    }

    /// Load config from the default Claude Desktop location
    pub fn load() -> ForgeResult<Self> {
        let path = Self::get_config_path()?;

        if !path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&path)?;
        let config: ClaudeConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save config to the default Claude Desktop location
    pub fn save(&self) -> ForgeResult<()> {
        let path = Self::get_config_path()?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;

        Ok(())
    }

    /// Get the Claude Desktop config path based on OS
    pub fn get_config_path() -> ForgeResult<PathBuf> {
        let config_dir = if cfg!(target_os = "macos") {
            dirs::home_dir()
                .ok_or(ForgeError::ClaudeDesktopNotFound)?
                .join("Library")
                .join("Application Support")
                .join("Claude")
        } else if cfg!(target_os = "windows") || cfg!(target_os = "linux") {
            dirs::config_dir()
                .ok_or(ForgeError::ClaudeDesktopNotFound)?
                .join("Claude")
        } else {
            return Err(ForgeError::Config(
                "Unsupported operating system".to_string(),
            ));
        };

        Ok(config_dir.join("claude_desktop_config.json"))
    }

    /// Add an MCP server to the configuration
    pub fn add_server(&mut self, name: String, server: McpServer) {
        self.mcpServers.insert(name, server);
    }

    /// Remove an MCP server from the configuration
    pub fn remove_server(&mut self, name: &str) -> ForgeResult<()> {
        if self.mcpServers.remove(name).is_none() {
            return Err(ForgeError::ServerNotFound(name.to_string()));
        }
        Ok(())
    }

    /// Get an MCP server by name
    pub fn get_server(&self, name: &str) -> Option<&McpServer> {
        self.mcpServers.get(name)
    }

    /// List all MCP server names
    pub fn list_servers(&self) -> Vec<String> {
        self.mcpServers.keys().cloned().collect()
    }

    /// Check if a server exists
    pub fn has_server(&self, name: &str) -> bool {
        self.mcpServers.contains_key(name)
    }

    /// Validate the configuration
    pub fn validate(&self) -> ForgeResult<Vec<String>> {
        let mut issues = Vec::new();

        for (name, server) in &self.mcpServers {
            // Check if command is empty
            if server.command.is_empty() {
                issues.push(format!("Server '{}' has empty command", name));
            }

            // Check for duplicate servers (same command and args)
            let mut seen = std::collections::HashSet::new();
            for (other_name, other_server) in &self.mcpServers {
                if name != other_name
                    && server.command == other_server.command
                    && server.args == other_server.args
                {
                    let key = format!("{}:{:?}", server.command, server.args);
                    if seen.insert(key) {
                        issues.push(format!(
                            "Servers '{}' and '{}' have identical configurations",
                            name, other_name
                        ));
                    }
                }
            }
        }

        Ok(issues)
    }
}

impl Default for ClaudeConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Forge-specific configuration (stored locally in .forge/)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeConfig {
    pub api_token: Option<String>,
    pub api_url: String,

    #[serde(default)]
    pub cache_dir: Option<PathBuf>,
}

impl ForgeConfig {
    pub fn new() -> Self {
        Self {
            api_token: None,
            api_url: "https://api.useforge.cc".to_string(),
            cache_dir: None,
        }
    }

    pub fn load() -> ForgeResult<Self> {
        let path = Self::get_config_path()?;

        if !path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(&path)?;
        let config: ForgeConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> ForgeResult<()> {
        let path = Self::get_config_path()?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)?;

        Ok(())
    }

    pub fn get_config_path() -> ForgeResult<PathBuf> {
        let config_dir = dirs::home_dir()
            .ok_or(ForgeError::Config(
                "Could not find home directory".to_string(),
            ))?
            .join(".forge");

        Ok(config_dir.join("config.json"))
    }

    #[allow(dead_code)]
    pub fn get_cache_dir() -> ForgeResult<PathBuf> {
        let cache_dir = dirs::home_dir()
            .ok_or(ForgeError::Config(
                "Could not find home directory".to_string(),
            ))?
            .join(".forge")
            .join("cache");

        fs::create_dir_all(&cache_dir)?;
        Ok(cache_dir)
    }

    #[allow(dead_code)]
    pub fn is_authenticated(&self) -> bool {
        self.api_token.is_some()
    }
}

impl Default for ForgeConfig {
    fn default() -> Self {
        Self::new()
    }
}
