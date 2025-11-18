use crate::colony::plugin::PluginManager;
use crate::error::ColonyResult;
use crate::utils;
use colored::Colorize;

/// List all installed plugins
pub fn list_plugins() -> ColonyResult<()> {
    let plugins_dir = PluginManager::default_plugins_dir()?;
    let mut manager = PluginManager::new(plugins_dir);

    // Discover plugins
    manager.discover_plugins()?;

    let plugins = manager.list_plugins();

    if plugins.is_empty() {
        utils::info("No plugins installed");
        println!();
        println!("  Plugins are located in: ~/.colony/plugins/");
        println!("  Each plugin should have a plugin.yaml manifest file.");
        return Ok(());
    }

    utils::header("Installed Plugins");
    println!();

    for plugin in plugins {
        let status = if plugin.enabled {
            "enabled".green()
        } else {
            "disabled".dimmed()
        };

        println!("  {} {} ({})", plugin.name().bold(), plugin.version().dimmed(), status);

        if let Some(desc) = &plugin.manifest.description {
            println!("    {}", desc.dimmed());
        }

        println!("    Type: {} | Path: {}", plugin.plugin_type(), plugin.path.display());

        if let Some(author) = &plugin.manifest.author {
            println!("    Author: {}", author.dimmed());
        }

        println!();
    }

    Ok(())
}

/// Show plugin details
pub fn show_plugin(name: &str) -> ColonyResult<()> {
    let plugins_dir = PluginManager::default_plugins_dir()?;
    let mut manager = PluginManager::new(plugins_dir);

    manager.discover_plugins()?;

    let plugin = manager.get_plugin(name).ok_or_else(|| {
        crate::error::ColonyError::Colony(format!("Plugin not found: {}", name))
    })?;

    utils::header(&format!("Plugin: {}", plugin.name()));
    println!();

    println!("Version: {}", plugin.version());
    println!("Type: {}", plugin.plugin_type());
    println!("Status: {}", if plugin.enabled {
        "enabled".green()
    } else {
        "disabled".dimmed()
    });
    println!();

    if let Some(desc) = &plugin.manifest.description {
        println!("Description:");
        println!("  {}", desc);
        println!();
    }

    if let Some(author) = &plugin.manifest.author {
        println!("Author: {}", author);
    }

    println!("Path: {}", plugin.path.display());
    println!("Installed: {}", plugin.installed_at.format("%Y-%m-%d %H:%M:%S"));
    println!();

    if let Some(hooks) = &plugin.manifest.hooks {
        println!("Hooks:");
        for hook in hooks {
            println!("  - {}", hook);
        }
        println!();
    }

    if let Some(deps) = &plugin.manifest.dependencies {
        println!("Dependencies:");
        for dep in deps {
            println!("  - {}", dep);
        }
        println!();
    }

    Ok(())
}

/// Enable a plugin
pub fn enable_plugin(name: &str) -> ColonyResult<()> {
    let plugins_dir = PluginManager::default_plugins_dir()?;
    let mut manager = PluginManager::new(plugins_dir);

    manager.discover_plugins()?;
    manager.enable_plugin(name)?;

    utils::success(&format!("Enabled plugin: {}", name));
    println!();
    println!("Note: Plugin changes will take effect on next colony start.");

    Ok(())
}

/// Disable a plugin
pub fn disable_plugin(name: &str) -> ColonyResult<()> {
    let plugins_dir = PluginManager::default_plugins_dir()?;
    let mut manager = PluginManager::new(plugins_dir);

    manager.discover_plugins()?;
    manager.disable_plugin(name)?;

    utils::success(&format!("Disabled plugin: {}", name));
    println!();
    println!("Note: Plugin changes will take effect on next colony start.");

    Ok(())
}
