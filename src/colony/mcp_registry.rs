use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::colony::config::McpServerConfig;

/// A registry of commonly used MCP servers with descriptions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    /// Unique identifier for this MCP server
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what this server does
    pub description: String,
    /// Category (filesystem, web, database, ai, etc.)
    pub category: String,
    /// Configuration template
    pub config: McpServerConfig,
    /// Prerequisites or notes
    pub notes: Option<String>,
}

/// Category of MCP servers
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum McpCategory {
    Filesystem,
    Web,
    Database,
    AI,
    Development,
    Productivity,
    Data,
    Other,
}

impl McpCategory {
    pub fn as_str(&self) -> &str {
        match self {
            McpCategory::Filesystem => "Filesystem",
            McpCategory::Web => "Web",
            McpCategory::Database => "Database",
            McpCategory::AI => "AI",
            McpCategory::Development => "Development",
            McpCategory::Productivity => "Productivity",
            McpCategory::Data => "Data",
            McpCategory::Other => "Other",
        }
    }
}

/// Registry of common MCP servers
pub struct McpRegistry;

impl McpRegistry {
    /// Get all available MCP servers
    pub fn all() -> Vec<McpServer> {
        vec![
            // Filesystem
            McpServer {
                id: "filesystem".to_string(),
                name: "Filesystem".to_string(),
                description: "Direct filesystem access for reading and writing files".to_string(),
                category: "Filesystem".to_string(),
                config: McpServerConfig {
                    command: "npx".to_string(),
                    args: Some(vec![
                        "-y".to_string(),
                        "@modelcontextprotocol/server-filesystem".to_string(),
                        "/tmp".to_string(),
                    ]),
                    env: None,
                },
                notes: Some("Change /tmp to your desired directory".to_string()),
            },

            // Web fetch
            McpServer {
                id: "fetch".to_string(),
                name: "Web Fetch".to_string(),
                description: "Fetch and retrieve content from web URLs".to_string(),
                category: "Web".to_string(),
                config: McpServerConfig {
                    command: "npx".to_string(),
                    args: Some(vec![
                        "-y".to_string(),
                        "@modelcontextprotocol/server-fetch".to_string(),
                    ]),
                    env: None,
                },
                notes: None,
            },

            // Brave Search
            McpServer {
                id: "brave-search".to_string(),
                name: "Brave Search".to_string(),
                description: "Search the web using Brave Search API".to_string(),
                category: "Web".to_string(),
                config: McpServerConfig {
                    command: "npx".to_string(),
                    args: Some(vec![
                        "-y".to_string(),
                        "@modelcontextprotocol/server-brave-search".to_string(),
                    ]),
                    env: Some({
                        let mut env = HashMap::new();
                        env.insert("BRAVE_API_KEY".to_string(), "your-api-key".to_string());
                        env
                    }),
                },
                notes: Some("Requires BRAVE_API_KEY environment variable".to_string()),
            },

            // GitHub
            McpServer {
                id: "github".to_string(),
                name: "GitHub".to_string(),
                description: "Interact with GitHub repositories, issues, and PRs".to_string(),
                category: "Development".to_string(),
                config: McpServerConfig {
                    command: "npx".to_string(),
                    args: Some(vec![
                        "-y".to_string(),
                        "@modelcontextprotocol/server-github".to_string(),
                    ]),
                    env: Some({
                        let mut env = HashMap::new();
                        env.insert("GITHUB_TOKEN".to_string(), "your-github-token".to_string());
                        env
                    }),
                },
                notes: Some("Requires GITHUB_TOKEN environment variable".to_string()),
            },

            // PostgreSQL
            McpServer {
                id: "postgres".to_string(),
                name: "PostgreSQL".to_string(),
                description: "Query and manage PostgreSQL databases".to_string(),
                category: "Database".to_string(),
                config: McpServerConfig {
                    command: "npx".to_string(),
                    args: Some(vec![
                        "-y".to_string(),
                        "@modelcontextprotocol/server-postgres".to_string(),
                    ]),
                    env: Some({
                        let mut env = HashMap::new();
                        env.insert("POSTGRES_CONNECTION_STRING".to_string(),
                                 "postgresql://user:password@localhost:5432/dbname".to_string());
                        env
                    }),
                },
                notes: Some("Requires POSTGRES_CONNECTION_STRING environment variable".to_string()),
            },

            // SQLite
            McpServer {
                id: "sqlite".to_string(),
                name: "SQLite".to_string(),
                description: "Query and manage SQLite databases".to_string(),
                category: "Database".to_string(),
                config: McpServerConfig {
                    command: "npx".to_string(),
                    args: Some(vec![
                        "-y".to_string(),
                        "@modelcontextprotocol/server-sqlite".to_string(),
                        "database.db".to_string(),
                    ]),
                    env: None,
                },
                notes: Some("Change database.db to your database file path".to_string()),
            },

            // Puppeteer (browser automation)
            McpServer {
                id: "puppeteer".to_string(),
                name: "Puppeteer".to_string(),
                description: "Browser automation for web scraping and testing".to_string(),
                category: "Web".to_string(),
                config: McpServerConfig {
                    command: "npx".to_string(),
                    args: Some(vec![
                        "-y".to_string(),
                        "@modelcontextprotocol/server-puppeteer".to_string(),
                    ]),
                    env: None,
                },
                notes: None,
            },

            // Memory
            McpServer {
                id: "memory".to_string(),
                name: "Memory".to_string(),
                description: "Persistent key-value storage for agent memory".to_string(),
                category: "Productivity".to_string(),
                config: McpServerConfig {
                    command: "npx".to_string(),
                    args: Some(vec![
                        "-y".to_string(),
                        "@modelcontextprotocol/server-memory".to_string(),
                    ]),
                    env: None,
                },
                notes: None,
            },

            // Sequential Thinking
            McpServer {
                id: "sequential-thinking".to_string(),
                name: "Sequential Thinking".to_string(),
                description: "Extended thinking and reasoning capabilities".to_string(),
                category: "AI".to_string(),
                config: McpServerConfig {
                    command: "npx".to_string(),
                    args: Some(vec![
                        "-y".to_string(),
                        "@modelcontextprotocol/server-sequential-thinking".to_string(),
                    ]),
                    env: None,
                },
                notes: None,
            },

            // Google Drive
            McpServer {
                id: "gdrive".to_string(),
                name: "Google Drive".to_string(),
                description: "Access and manage Google Drive files".to_string(),
                category: "Productivity".to_string(),
                config: McpServerConfig {
                    command: "npx".to_string(),
                    args: Some(vec![
                        "-y".to_string(),
                        "@modelcontextprotocol/server-gdrive".to_string(),
                    ]),
                    env: Some({
                        let mut env = HashMap::new();
                        env.insert("GOOGLE_APPLICATION_CREDENTIALS".to_string(),
                                 "path/to/credentials.json".to_string());
                        env
                    }),
                },
                notes: Some("Requires Google Cloud credentials".to_string()),
            },

            // Slack
            McpServer {
                id: "slack".to_string(),
                name: "Slack".to_string(),
                description: "Send messages and interact with Slack".to_string(),
                category: "Productivity".to_string(),
                config: McpServerConfig {
                    command: "npx".to_string(),
                    args: Some(vec![
                        "-y".to_string(),
                        "@modelcontextprotocol/server-slack".to_string(),
                    ]),
                    env: Some({
                        let mut env = HashMap::new();
                        env.insert("SLACK_BOT_TOKEN".to_string(), "xoxb-your-token".to_string());
                        env
                    }),
                },
                notes: Some("Requires SLACK_BOT_TOKEN environment variable".to_string()),
            },

            // EverArt (image generation)
            McpServer {
                id: "everart".to_string(),
                name: "EverArt".to_string(),
                description: "AI image generation via EverArt".to_string(),
                category: "AI".to_string(),
                config: McpServerConfig {
                    command: "npx".to_string(),
                    args: Some(vec![
                        "-y".to_string(),
                        "@modelcontextprotocol/server-everart".to_string(),
                    ]),
                    env: Some({
                        let mut env = HashMap::new();
                        env.insert("EVERART_API_KEY".to_string(), "your-api-key".to_string());
                        env
                    }),
                },
                notes: Some("Requires EVERART_API_KEY environment variable".to_string()),
            },

            // Google Maps
            McpServer {
                id: "google-maps".to_string(),
                name: "Google Maps".to_string(),
                description: "Access Google Maps API for location data".to_string(),
                category: "Data".to_string(),
                config: McpServerConfig {
                    command: "npx".to_string(),
                    args: Some(vec![
                        "-y".to_string(),
                        "@modelcontextprotocol/server-google-maps".to_string(),
                    ]),
                    env: Some({
                        let mut env = HashMap::new();
                        env.insert("GOOGLE_MAPS_API_KEY".to_string(), "your-api-key".to_string());
                        env
                    }),
                },
                notes: Some("Requires GOOGLE_MAPS_API_KEY environment variable".to_string()),
            },
        ]
    }

    /// Get MCP servers by category
    pub fn by_category(category: &str) -> Vec<McpServer> {
        Self::all()
            .into_iter()
            .filter(|s| s.category == category)
            .collect()
    }

    /// Get a specific MCP server by ID
    pub fn get(id: &str) -> Option<McpServer> {
        Self::all().into_iter().find(|s| s.id == id)
    }

    /// Get all unique categories
    pub fn categories() -> Vec<String> {
        let mut categories: Vec<String> = Self::all()
            .into_iter()
            .map(|s| s.category)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        categories.sort();
        categories
    }

    /// Get common MCP servers for specific use cases
    pub fn for_executor() -> Vec<McpServer> {
        // Recommended servers for MCP executor
        vec!["filesystem", "fetch", "puppeteer", "memory"]
            .into_iter()
            .filter_map(|id| Self::get(id))
            .collect()
    }

    pub fn for_web_development() -> Vec<McpServer> {
        vec!["filesystem", "fetch", "puppeteer", "github"]
            .into_iter()
            .filter_map(|id| Self::get(id))
            .collect()
    }

    pub fn for_data_analysis() -> Vec<McpServer> {
        vec!["filesystem", "postgres", "sqlite", "fetch"]
            .into_iter()
            .filter_map(|id| Self::get(id))
            .collect()
    }

    pub fn for_automation() -> Vec<McpServer> {
        vec!["filesystem", "fetch", "slack", "github", "puppeteer"]
            .into_iter()
            .filter_map(|id| Self::get(id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_all() {
        let servers = McpRegistry::all();
        assert!(!servers.is_empty());
        assert!(servers.iter().any(|s| s.id == "filesystem"));
        assert!(servers.iter().any(|s| s.id == "fetch"));
    }

    #[test]
    fn test_registry_by_category() {
        let web_servers = McpRegistry::by_category("Web");
        assert!(!web_servers.is_empty());
        assert!(web_servers.iter().all(|s| s.category == "Web"));
    }

    #[test]
    fn test_registry_get() {
        let server = McpRegistry::get("filesystem");
        assert!(server.is_some());
        assert_eq!(server.unwrap().id, "filesystem");
    }

    #[test]
    fn test_for_executor() {
        let servers = McpRegistry::for_executor();
        assert!(!servers.is_empty());
        assert!(servers.iter().any(|s| s.id == "filesystem"));
    }
}
