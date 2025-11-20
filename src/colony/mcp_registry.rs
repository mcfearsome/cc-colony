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

            // Code Index
            McpServer {
                id: "code-index".to_string(),
                name: "Code Index".to_string(),
                description: "Index and search codebase with semantic code understanding".to_string(),
                category: "Development".to_string(),
                config: McpServerConfig {
                    command: "npx".to_string(),
                    args: Some(vec![
                        "-y".to_string(),
                        "code-index-mcp".to_string(),
                    ]),
                    env: None,
                },
                notes: Some("Provides semantic code search and navigation".to_string()),
            },

            // Figma
            McpServer {
                id: "figma".to_string(),
                name: "Figma".to_string(),
                description: "Access and interact with Figma designs and components".to_string(),
                category: "Development".to_string(),
                config: McpServerConfig {
                    command: "npx".to_string(),
                    args: Some(vec![
                        "-y".to_string(),
                        "@modelcontextprotocol/server-figma".to_string(),
                    ]),
                    env: Some({
                        let mut env = HashMap::new();
                        env.insert("FIGMA_ACCESS_TOKEN".to_string(), "your-figma-token".to_string());
                        env
                    }),
                },
                notes: Some("Requires FIGMA_ACCESS_TOKEN environment variable".to_string()),
            },

            // Desktop Commander
            McpServer {
                id: "desktop-commander".to_string(),
                name: "Desktop Commander".to_string(),
                description: "Automate desktop applications and system interactions".to_string(),
                category: "Productivity".to_string(),
                config: McpServerConfig {
                    command: "npx".to_string(),
                    args: Some(vec![
                        "-y".to_string(),
                        "desktop-commander-mcp".to_string(),
                    ]),
                    env: None,
                },
                notes: Some("Provides desktop automation capabilities (use with caution)".to_string()),
            },

            // Serena
            McpServer {
                id: "serena".to_string(),
                name: "Serena".to_string(),
                description: "Intelligent file search and workspace navigation".to_string(),
                category: "Productivity".to_string(),
                config: McpServerConfig {
                    command: "npx".to_string(),
                    args: Some(vec![
                        "-y".to_string(),
                        "serena-mcp".to_string(),
                    ]),
                    env: None,
                },
                notes: Some("Enhanced file finding with semantic understanding".to_string()),
            },

            // Context7
            McpServer {
                id: "context7".to_string(),
                name: "Context7".to_string(),
                description: "Advanced context management and retrieval for AI workflows".to_string(),
                category: "AI".to_string(),
                config: McpServerConfig {
                    command: "npx".to_string(),
                    args: Some(vec![
                        "-y".to_string(),
                        "context7-mcp".to_string(),
                    ]),
                    env: None,
                },
                notes: Some("Manages conversation context and knowledge retrieval".to_string()),
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

    /// Detect overlapping functionality between MCP servers
    /// Returns warnings about potential conflicts or redundancy
    pub fn detect_overlaps(server_ids: &[String]) -> Vec<String> {
        let mut warnings = Vec::new();

        // Define overlap groups (servers that provide similar functionality)
        let overlap_groups = vec![
            (
                vec!["filesystem", "serena"],
                "Both provide file access. 'serena' adds semantic search on top of basic filesystem access.",
            ),
            (
                vec!["filesystem", "gdrive"],
                "Both provide file storage access. Consider which storage location you need.",
            ),
            (
                vec!["memory", "context7"],
                "Both provide context/memory management. 'context7' is more advanced for AI workflows.",
            ),
            (
                vec!["fetch", "puppeteer"],
                "Both can retrieve web content. 'puppeteer' provides browser automation, 'fetch' is simpler for API calls.",
            ),
            (
                vec!["desktop-commander", "puppeteer"],
                "Both provide automation. 'puppeteer' is web-focused, 'desktop-commander' is for desktop apps.",
            ),
            (
                vec!["sequential-thinking", "context7"],
                "Both enhance AI reasoning. May be redundant unless you need both specialized capabilities.",
            ),
            (
                vec!["postgres", "sqlite"],
                "Both provide database access. Choose based on your database type.",
            ),
        ];

        // Check for overlaps
        for (group, warning) in overlap_groups {
            let matching: Vec<_> = group
                .iter()
                .filter(|id| server_ids.contains(&id.to_string()))
                .collect();

            if matching.len() > 1 {
                warnings.push(format!(
                    "⚠️  Overlap detected: {} - {}",
                    matching
                        .iter()
                        .map(|s| format!("'{}'", s))
                        .collect::<Vec<_>>()
                        .join(", "),
                    warning
                ));
            }
        }

        // Check for too many servers (performance warning)
        if server_ids.len() > 8 {
            warnings.push(format!(
                "⚠️  Performance: {} MCP servers configured. Consider if all are needed (each adds overhead).",
                server_ids.len()
            ));
        }

        warnings
    }

    /// Suggest complementary servers based on selected servers
    pub fn suggest_complementary(server_ids: &[String]) -> Vec<(String, String)> {
        let mut suggestions = Vec::new();

        // Common pairings
        if server_ids.contains(&"github".to_string()) && !server_ids.contains(&"code-index".to_string()) {
            suggestions.push((
                "code-index".to_string(),
                "Adds semantic code search to complement GitHub integration".to_string(),
            ));
        }

        if server_ids.contains(&"figma".to_string()) && !server_ids.contains(&"puppeteer".to_string()) {
            suggestions.push((
                "puppeteer".to_string(),
                "Can automate Figma workflows in browser".to_string(),
            ));
        }

        if (server_ids.contains(&"postgres".to_string()) || server_ids.contains(&"sqlite".to_string()))
            && !server_ids.contains(&"filesystem".to_string())
        {
            suggestions.push((
                "filesystem".to_string(),
                "Useful for database backups and schema file access".to_string(),
            ));
        }

        if server_ids.contains(&"slack".to_string()) && !server_ids.contains(&"fetch".to_string()) {
            suggestions.push((
                "fetch".to_string(),
                "Enables fetching data from APIs to send to Slack".to_string(),
            ));
        }

        if server_ids.contains(&"code-index".to_string()) && !server_ids.contains(&"filesystem".to_string()) {
            suggestions.push((
                "filesystem".to_string(),
                "Required for code-index to access source files".to_string(),
            ));
        }

        suggestions
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

    #[test]
    fn test_detect_overlaps() {
        // Test filesystem + serena overlap
        let server_ids = vec!["filesystem".to_string(), "serena".to_string()];
        let warnings = McpRegistry::detect_overlaps(&server_ids);
        assert!(!warnings.is_empty());
        assert!(warnings[0].contains("filesystem"));
        assert!(warnings[0].contains("serena"));

        // Test no overlaps
        let server_ids = vec!["filesystem".to_string(), "github".to_string()];
        let warnings = McpRegistry::detect_overlaps(&server_ids);
        assert!(warnings.is_empty());

        // Test performance warning
        let server_ids: Vec<String> = (0..10).map(|i| format!("server-{}", i)).collect();
        let warnings = McpRegistry::detect_overlaps(&server_ids);
        assert!(warnings.iter().any(|w| w.contains("Performance")));
    }

    #[test]
    fn test_suggest_complementary() {
        // Test github suggests code-index
        let server_ids = vec!["github".to_string()];
        let suggestions = McpRegistry::suggest_complementary(&server_ids);
        assert!(suggestions.iter().any(|(id, _)| id == "code-index"));

        // Test code-index suggests filesystem
        let server_ids = vec!["code-index".to_string()];
        let suggestions = McpRegistry::suggest_complementary(&server_ids);
        assert!(suggestions.iter().any(|(id, _)| id == "filesystem"));
    }

    #[test]
    fn test_new_servers_exist() {
        // Verify the newly added servers are in the registry
        assert!(McpRegistry::get("code-index").is_some());
        assert!(McpRegistry::get("figma").is_some());
        assert!(McpRegistry::get("desktop-commander").is_some());
        assert!(McpRegistry::get("serena").is_some());
        assert!(McpRegistry::get("context7").is_some());
    }
}
