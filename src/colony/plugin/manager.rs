use super::types::{Plugin, PluginManifest};
use crate::error::{ColonyError, ColonyResult};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Plugin manager - handles plugin discovery, loading, and lifecycle
pub struct PluginManager {
    plugins_dir: PathBuf,
    plugins: HashMap<String, Plugin>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new(plugins_dir: PathBuf) -> Self {
        Self {
            plugins_dir,
            plugins: HashMap::new(),
        }
    }

    /// Get default plugins directory (~/.colony/plugins/)
    pub fn default_plugins_dir() -> ColonyResult<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| ColonyError::Colony("Could not determine home directory".to_string()))?;
        Ok(home.join(".colony").join("plugins"))
    }

    /// Initialize plugins directory
    pub fn initialize(&self) -> ColonyResult<()> {
        fs::create_dir_all(&self.plugins_dir).map_err(|e| {
            ColonyError::Colony(format!("Failed to create plugins directory: {}", e))
        })?;
        Ok(())
    }

    /// Discover and load all plugins from the plugins directory
    pub fn discover_plugins(&mut self) -> ColonyResult<()> {
        if !self.plugins_dir.exists() {
            return Ok(());
        }

        let entries = fs::read_dir(&self.plugins_dir).map_err(|e| {
            ColonyError::Colony(format!("Failed to read plugins directory: {}", e))
        })?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(plugin) = self.load_plugin(&path) {
                    self.plugins.insert(plugin.name().to_string(), plugin);
                }
            }
        }

        Ok(())
    }

    /// Load a single plugin from a directory
    fn load_plugin(&self, plugin_dir: &Path) -> ColonyResult<Plugin> {
        let manifest_path = plugin_dir.join("plugin.yaml");

        if !manifest_path.exists() {
            return Err(ColonyError::Colony(format!(
                "Plugin manifest not found: {}",
                manifest_path.display()
            )));
        }

        let content = fs::read_to_string(&manifest_path).map_err(|e| {
            ColonyError::Colony(format!("Failed to read plugin manifest: {}", e))
        })?;

        let manifest: PluginManifest = serde_yaml::from_str(&content).map_err(|e| {
            ColonyError::Colony(format!("Failed to parse plugin manifest: {}", e))
        })?;

        Ok(Plugin::new(manifest, plugin_dir.to_path_buf()))
    }

    /// Get a plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<&Plugin> {
        self.plugins.get(name)
    }

    /// Get a mutable plugin by name
    pub fn get_plugin_mut(&mut self, name: &str) -> Option<&mut Plugin> {
        self.plugins.get_mut(name)
    }

    /// List all plugins
    pub fn list_plugins(&self) -> Vec<&Plugin> {
        let mut plugins: Vec<&Plugin> = self.plugins.values().collect();
        plugins.sort_by_key(|p| &p.manifest.name);
        plugins
    }

    /// Enable a plugin
    pub fn enable_plugin(&mut self, name: &str) -> ColonyResult<()> {
        let plugin = self
            .get_plugin_mut(name)
            .ok_or_else(|| ColonyError::Colony(format!("Plugin not found: {}", name)))?;

        plugin.enabled = true;
        Ok(())
    }

    /// Disable a plugin
    pub fn disable_plugin(&mut self, name: &str) -> ColonyResult<()> {
        let plugin = self
            .get_plugin_mut(name)
            .ok_or_else(|| ColonyError::Colony(format!("Plugin not found: {}", name)))?;

        plugin.enabled = false;
        Ok(())
    }

    /// Get enabled plugins
    pub fn enabled_plugins(&self) -> Vec<&Plugin> {
        self.plugins
            .values()
            .filter(|p| p.enabled)
            .collect()
    }
}
