use crate::colony::template::{get_builtin_templates, TemplateManager};
use crate::error::ColonyResult;
use crate::utils;
use colored::Colorize;
use std::fs;

/// List all available templates
pub fn list_templates() -> ColonyResult<()> {
    let templates_dir = TemplateManager::default_templates_dir()?;
    let builtin_dir = TemplateManager::builtin_templates_dir();
    let mut manager = TemplateManager::new(templates_dir, builtin_dir);

    // Discover templates
    manager.discover_templates()?;

    let templates = manager.list_templates();

    if templates.is_empty() {
        // Show built-in templates even if not installed
        utils::info("No templates installed. Available built-in templates:");
        println!();

        for (name, _) in get_builtin_templates() {
            println!("  {} (built-in)", name.bold());
        }

        println!();
        println!("Use 'colony template install <name>' to install a built-in template");
        println!("Or create custom templates in .colony/templates/");
        return Ok(());
    }

    utils::header("Available Templates");
    println!();

    for template in templates {
        let source = if template.is_builtin {
            "built-in".dimmed()
        } else {
            "custom".cyan()
        };

        println!(
            "  {} {} ({})",
            template.name().bold(),
            template.version().dimmed(),
            source
        );

        if let Some(desc) = &template.template.description {
            println!("    {}", desc.dimmed());
        }

        println!("    Role: {}", template.template.agent.role);
        println!();
    }

    Ok(())
}

/// Show template details
pub fn show_template(name: &str) -> ColonyResult<()> {
    let templates_dir = TemplateManager::default_templates_dir()?;
    let builtin_dir = TemplateManager::builtin_templates_dir();
    let mut manager = TemplateManager::new(templates_dir, builtin_dir);

    manager.discover_templates()?;

    let template_meta = manager.get_template(name).ok_or_else(|| {
        crate::error::ColonyError::Colony(format!("Template not found: {}", name))
    })?;

    let template = &template_meta.template;

    utils::header(&format!("Template: {}", template.name));
    println!();

    println!("Version: {}", template.version);
    println!(
        "Source: {}",
        if template_meta.is_builtin {
            "built-in"
        } else {
            "custom"
        }
    );
    println!();

    if let Some(desc) = &template.description {
        println!("Description:");
        println!("  {}", desc);
        println!();
    }

    if let Some(author) = &template.author {
        println!("Author: {}", author);
    }

    if let Some(license) = &template.license {
        println!("License: {}", license);
    }

    println!();
    println!("Agent Configuration:");
    println!("  Role: {}", template.agent.role);
    println!("  Focus: {}", template.agent.focus);
    println!("  Model: {}", template.agent.model);
    println!();

    if let Some(reqs) = &template.requirements {
        if let Some(repo_types) = &reqs.repo_types {
            println!("Compatible Repository Types:");
            for rt in repo_types {
                println!("  - {}", rt);
            }
            println!();
        }
    }

    Ok(())
}

/// Install a built-in template to .colony/templates/
pub fn install_template(name: &str) -> ColonyResult<()> {
    // Find the built-in template
    let builtin = get_builtin_templates()
        .into_iter()
        .find(|(n, _)| n == &name)
        .ok_or_else(|| {
            crate::error::ColonyError::Colony(format!("Built-in template not found: {}", name))
        })?;

    // Create templates directory
    let templates_dir = TemplateManager::default_templates_dir()?;
    fs::create_dir_all(&templates_dir).map_err(|e| {
        crate::error::ColonyError::Colony(format!("Failed to create templates directory: {}", e))
    })?;

    // Create template directory
    let template_dir = templates_dir.join(name);
    if template_dir.exists() {
        return Err(crate::error::ColonyError::Colony(format!(
            "Template '{}' is already installed",
            name
        )));
    }

    fs::create_dir_all(&template_dir).map_err(|e| {
        crate::error::ColonyError::Colony(format!("Failed to create template directory: {}", e))
    })?;

    // Write template.yaml
    let template_file = template_dir.join("template.yaml");
    fs::write(&template_file, builtin.1).map_err(|e| {
        crate::error::ColonyError::Colony(format!("Failed to write template file: {}", e))
    })?;

    utils::success(&format!("Installed template: {}", name));
    println!();
    println!("Template location: {}", template_dir.display());
    println!();
    println!("You can now use this template when creating agents:");
    println!("  colony agent create --template {} --id my-agent", name);

    Ok(())
}

/// List built-in templates available for installation
pub fn list_builtin() -> ColonyResult<()> {
    utils::header("Built-in Templates");
    println!();

    for (name, content) in get_builtin_templates() {
        // Parse to get description
        let template: crate::colony::template::AgentTemplate = serde_yaml::from_str(content)
            .unwrap_or_else(|_| panic!("Failed to parse built-in template: {}", name));

        println!("  {}", name.bold());
        if let Some(desc) = &template.description {
            println!("    {}", desc.dimmed());
        }
        println!("    Role: {}", template.agent.role);
        println!();
    }

    println!("Use 'colony template install <name>' to install a template");
    println!("Use 'colony template show <name>' to see details (after installing)");

    Ok(())
}
