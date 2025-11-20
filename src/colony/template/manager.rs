use super::types::{AgentTemplate, TemplateMetadata};
use crate::error::{ColonyError, ColonyResult};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Template manager - handles template discovery and loading
pub struct TemplateManager {
    templates_dir: PathBuf,
    builtin_templates_dir: Option<PathBuf>,
    templates: HashMap<String, TemplateMetadata>,
}

impl TemplateManager {
    /// Create a new template manager
    pub fn new(templates_dir: PathBuf, builtin_templates_dir: Option<PathBuf>) -> Self {
        Self {
            templates_dir,
            builtin_templates_dir,
            templates: HashMap::new(),
        }
    }

    /// Get default templates directory
    pub fn default_templates_dir() -> ColonyResult<PathBuf> {
        Ok(PathBuf::from(".colony/templates"))
    }

    /// Get builtin templates directory (embedded in binary or standard location)
    pub fn builtin_templates_dir() -> Option<PathBuf> {
        // For now, return None. In the future, we could embed templates
        // or install them to a standard location
        None
    }

    /// Initialize templates directory
    pub fn initialize(&self) -> ColonyResult<()> {
        fs::create_dir_all(&self.templates_dir).map_err(|e| {
            ColonyError::Colony(format!("Failed to create templates directory: {}", e))
        })?;
        Ok(())
    }

    /// Discover and load all templates
    pub fn discover_templates(&mut self) -> ColonyResult<()> {
        // Load user templates
        let templates_dir = self.templates_dir.clone();
        if templates_dir.exists() {
            self.load_templates_from_dir(&templates_dir, false)?;
        }

        // Load builtin templates
        if let Some(builtin_dir) = self.builtin_templates_dir.clone() {
            if builtin_dir.exists() {
                self.load_templates_from_dir(&builtin_dir, true)?;
            }
        }

        Ok(())
    }

    /// Load templates from a directory
    fn load_templates_from_dir(&mut self, dir: &Path, is_builtin: bool) -> ColonyResult<()> {
        let entries = fs::read_dir(dir).map_err(|e| {
            ColonyError::Colony(format!("Failed to read templates directory: {}", e))
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(metadata) = self.load_template(&path, is_builtin) {
                    self.templates.insert(metadata.name().to_string(), metadata);
                }
            }
        }

        Ok(())
    }

    /// Load a single template from a directory
    fn load_template(
        &self,
        template_dir: &Path,
        is_builtin: bool,
    ) -> ColonyResult<TemplateMetadata> {
        let manifest_path = template_dir.join("template.yaml");

        if !manifest_path.exists() {
            return Err(ColonyError::Colony(format!(
                "Template manifest not found: {}",
                manifest_path.display()
            )));
        }

        let content = fs::read_to_string(&manifest_path)
            .map_err(|e| ColonyError::Colony(format!("Failed to read template manifest: {}", e)))?;

        let template: AgentTemplate = serde_yaml::from_str(&content).map_err(|e| {
            ColonyError::Colony(format!("Failed to parse template manifest: {}", e))
        })?;

        Ok(TemplateMetadata {
            template,
            path: template_dir.to_path_buf(),
            is_builtin,
        })
    }

    /// Get a template by name
    pub fn get_template(&self, name: &str) -> Option<&TemplateMetadata> {
        self.templates.get(name)
    }

    /// List all templates
    pub fn list_templates(&self) -> Vec<&TemplateMetadata> {
        let mut templates: Vec<&TemplateMetadata> = self.templates.values().collect();
        templates.sort_by_key(|t| &t.template.name);
        templates
    }

    /// List builtin templates
    pub fn list_builtin_templates(&self) -> Vec<&TemplateMetadata> {
        self.templates.values().filter(|t| t.is_builtin).collect()
    }

    /// List user templates
    pub fn list_user_templates(&self) -> Vec<&TemplateMetadata> {
        self.templates.values().filter(|t| !t.is_builtin).collect()
    }
}
