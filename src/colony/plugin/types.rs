use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Plugin type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginType {
    Backend,
    Ui,
    Tool,
}

impl std::fmt::Display for PluginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginType::Backend => write!(f, "backend"),
            PluginType::Ui => write!(f, "ui"),
            PluginType::Tool => write!(f, "tool"),
        }
    }
}

/// Plugin manifest (defined in plugin.yaml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub author: Option<String>,
    #[serde(rename = "type")]
    pub plugin_type: PluginType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<String>>,
}

/// Plugin metadata and state
#[derive(Debug, Clone)]
pub struct Plugin {
    pub manifest: PluginManifest,
    pub path: PathBuf,
    pub enabled: bool,
    pub installed_at: chrono::DateTime<chrono::Utc>,
}

impl Plugin {
    pub fn new(manifest: PluginManifest, path: PathBuf) -> Self {
        Self {
            manifest,
            path,
            enabled: false,
            installed_at: chrono::Utc::now(),
        }
    }

    pub fn name(&self) -> &str {
        &self.manifest.name
    }

    pub fn version(&self) -> &str {
        &self.manifest.version
    }

    pub fn plugin_type(&self) -> &PluginType {
        &self.manifest.plugin_type
    }
}

/// Plugin configuration in colony.yml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<HashMap<String, serde_json::Value>>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            config: None,
        }
    }
}
