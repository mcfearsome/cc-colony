use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTemplate {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub agent: AgentTemplateConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requirements: Option<TemplateRequirements>,
}

/// Agent configuration in template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTemplateConfig {
    pub role: String,
    pub focus: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub startup_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behavior: Option<BehaviorConfig>,
}

/// Behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    pub initiative_level: Option<String>,
    pub communication_style: Option<String>,
    pub thoroughness: Option<String>,
}

/// Template requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateRequirements {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<String>>,
}

/// Template metadata
#[derive(Debug, Clone)]
pub struct TemplateMetadata {
    pub template: AgentTemplate,
    pub path: std::path::PathBuf,
    pub is_builtin: bool,
}

impl TemplateMetadata {
    pub fn name(&self) -> &str {
        &self.template.name
    }

    pub fn version(&self) -> &str {
        &self.template.version
    }
}
