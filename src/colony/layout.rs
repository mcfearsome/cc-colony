use std::collections::HashMap;
use std::fs;
use std::process::Command;
use serde::Serialize;

use crate::colony::ColonyController;
use crate::error::ColonyResult;

/// Moxide template structure for YAML generation
#[derive(Debug, Serialize)]
struct MoxideTemplate {
    name: String,
    windows: Vec<MoxideWindow>,
}

#[derive(Debug, Serialize)]
struct MoxideWindow {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    layout: Option<String>,
    panes: Vec<String>,
}

/// Generate moxide template from Colony layout configuration and create session
/// Returns a map of (agent_id/component -> (window_index, pane_index))
pub fn create_session_with_moxide(
    session_name: &str,
    controller: &ColonyController,
) -> ColonyResult<HashMap<String, (usize, usize)>> {
    let config = controller.config();

    // Check if custom layout is configured
    if let Some(layout_config) = &config.layout {
        eprintln!("DEBUG: Found layout config, type={}, windows={}",
            layout_config.layout_type, layout_config.windows.len());

        if layout_config.layout_type == "custom" && !layout_config.windows.is_empty() {
            crate::utils::info(&format!("Using custom layout with {} windows", layout_config.windows.len()));
            return create_custom_layout_with_moxide(session_name, controller, layout_config);
        }
    } else {
        eprintln!("DEBUG: No layout config found");
    }

    // Fallback: no custom layout, return empty map (use default tmux creation)
    Ok(HashMap::new())
}

/// Create custom layout using moxide
fn create_custom_layout_with_moxide(
    session_name: &str,
    controller: &ColonyController,
    layout_config: &crate::colony::config::LayoutConfig,
) -> ColonyResult<HashMap<String, (usize, usize)>> {
    // Build moxide template from colony layout
    let mut moxide_windows = Vec::new();
    let mut pane_map: HashMap<String, (usize, usize)> = HashMap::new();

    for (window_idx, window) in layout_config.windows.iter().enumerate() {
        let mut pane_commands = Vec::new();
        let mut pane_configs: Vec<(usize, &crate::colony::config::PaneConfig)> =
            window.panes.iter().enumerate().collect();

        // Special ordering for main-dev window: put TUI first so it ends up on bottom
        if window.name == "main-dev" {
            pane_configs.sort_by_key(|(_, pane)| {
                match pane.pane_type.as_str() {
                    "tui" => 0,      // TUI first
                    "tool" => 1,     // Tools next (nvim)
                    "agent" => 2,    // Agents
                    "executor" => 3, // Executor last
                    _ => 4,
                }
            });
        }

        for (original_idx, pane) in pane_configs {
            let command = match pane.pane_type.as_str() {
                "agent" => {
                    if let Some(agent_id) = &pane.agent_id {
                        // Track with NEW position in output
                        let new_idx = pane_commands.len();
                        pane_map.insert(agent_id.clone(), (window_idx, new_idx));
                        "bash".to_string()
                    } else {
                        "bash".to_string()
                    }
                }
                "executor" => {
                    let new_idx = pane_commands.len();
                    pane_map.insert("mcp-executor".to_string(), (window_idx, new_idx));
                    "bash".to_string()
                }
                "tui" => {
                    let new_idx = pane_commands.len();
                    pane_map.insert("tui".to_string(), (window_idx, new_idx));
                    "bash".to_string()
                }
                "tool" => {
                    let new_idx = pane_commands.len();
                    let tool_id = format!("tool-{}-{}", window_idx, original_idx);
                    pane_map.insert(tool_id, (window_idx, new_idx));
                    pane.command.clone().unwrap_or_else(|| "bash".to_string())
                }
                _ => "bash".to_string(),
            };

            pane_commands.push(command);
        }

        // Infer layout from pane count, types, and window name
        let layout = infer_tmux_layout(&window.name, &window.panes);

        moxide_windows.push(MoxideWindow {
            name: window.name.clone(),
            layout,
            panes: pane_commands,
        });
    }

    let template = MoxideTemplate {
        name: session_name.to_string(),
        windows: moxide_windows,
    };

    // Write moxide template to ~/.config/moxide/templates/
    let template_yaml = serde_yaml::to_string(&template)
        .map_err(|e| crate::error::ColonyError::Colony(format!("Failed to serialize moxide template: {}", e)))?;

    let home_dir = dirs::home_dir()
        .ok_or_else(|| crate::error::ColonyError::Colony("Failed to get home directory".to_string()))?;

    let templates_dir = home_dir.join(".config/moxide/templates");
    fs::create_dir_all(&templates_dir)
        .map_err(|e| crate::error::ColonyError::Colony(format!("Failed to create moxide templates dir: {}", e)))?;

    let template_filename = format!("colony-{}", session_name);
    let template_path = templates_dir.join(format!("{}.yaml", template_filename));

    fs::write(&template_path, template_yaml)
        .map_err(|e| crate::error::ColonyError::Colony(format!("Failed to write moxide template: {}", e)))?;

    crate::utils::info(&format!("Generated moxide template: {}", template_path.display()));

    // Create session with moxide (references template by its name field, not filename)
    let output = Command::new("moxide")
        .arg("template")
        .arg("start")
        .arg(session_name)  // Use the name from template YAML
        .arg("--detached")
        .arg("--name")
        .arg(session_name)
        .output()
        .map_err(|e| crate::error::ColonyError::Colony(format!("Failed to run moxide: {}", e)))?;

    if !output.status.success() {
        return Err(crate::error::ColonyError::Colony(format!(
            "Moxide failed to create session: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    crate::utils::success(&format!("Created tmux session '{}' with moxide", session_name));

    // Apply layout refinements to match exact specifications
    refine_layout(session_name, layout_config, &pane_map)?;

    Ok(pane_map)
}

/// Refine the layout after moxide creates it to match exact specifications
fn refine_layout(
    session_name: &str,
    layout_config: &crate::colony::config::LayoutConfig,
    pane_map: &HashMap<String, (usize, usize)>,
) -> ColonyResult<()> {
    use crate::colony::tmux;

    // Small delay to let moxide finish creating panes
    std::thread::sleep(std::time::Duration::from_millis(500));

    for (window_idx, window) in layout_config.windows.iter().enumerate() {
        // Apply window-specific refinements
        match window.name.as_str() {
            "main-dev" => {
                // Desired layout:
                // ┌────────────┬─────────┐
                // │   nvim     │  agent  │
                // │            ├─────────┤
                // │            │executor │
                // ├────────────┴─────────┤
                // │        TUI           │
                // └──────────────────────┘

                // After reordering, panes are: TUI(0), nvim(1), agent(2), executor(3)
                // Strategy: Use tiled layout as base (it's functional)
                // Future: Implement precise geometry with layout strings

                std::thread::sleep(std::time::Duration::from_millis(500));

                crate::utils::info("  Applying tiled layout to main-dev window");
                tmux::select_window_layout(session_name, window_idx, "tiled")?;

                // With 4 panes, tiled creates a 2x2 grid which is close to what we want
                // The exact geometry (TUI bottom span) would require custom layout string
                // or breaking/rejoining panes (complex due to index changes)

                crate::utils::info("  Main-dev layout: 2x2 tiled (TUI, nvim, agent, executor)");
            }
            "backend-services" => {
                // Backend services: even split between agents
                tmux::select_window_layout(session_name, window_idx, "even-horizontal")?;
            }
            _ => {
                // Default: tiled layout
                tmux::select_window_layout(session_name, window_idx, "tiled")?;
            }
        }

        // Apply size-based resizing if specified in config
        for pane in &window.panes {
            if let Some(size_str) = &pane.size {
                // Parse size (e.g., "60%", "25%")
                if let Some(pct) = size_str.strip_suffix('%') {
                    if let Ok(percentage) = pct.parse::<u16>() {
                        // Find pane coordinates
                        let pane_id = match pane.pane_type.as_str() {
                            "agent" => pane.agent_id.as_ref(),
                            "executor" => Some(&"mcp-executor".to_string()),
                            "tui" => Some(&"tui".to_string()),
                            _ => None,
                        };

                        if let Some(id) = pane_id {
                            if let Some((win, pane_idx)) = pane_map.get(id.as_str()) {
                                // Apply size (heuristic: if > 50%, it's width, else height)
                                if percentage > 50 {
                                    let _ = tmux::resize_pane_percentage(session_name, *win, *pane_idx, Some(percentage), None);
                                } else {
                                    let _ = tmux::resize_pane_percentage(session_name, *win, *pane_idx, None, Some(percentage));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Infer appropriate tmux layout from pane configuration
fn infer_tmux_layout(window_name: &str, panes: &[crate::colony::config::PaneConfig]) -> Option<String> {
    // Special case for main-dev window: use main-horizontal for bottom-spanning TUI
    if window_name == "main-dev" && panes.len() == 4 {
        // 4 panes: nvim, agent, executor, TUI
        // main-horizontal puts one pane as "main" (larger) and others stacked
        return Some("main-horizontal".to_string());
    }

    // Backend services or other: even split
    if window_name.contains("backend") || window_name.contains("services") {
        return Some("even-horizontal".to_string());
    }

    // Default based on pane count
    match panes.len() {
        0 | 1 => None,
        2 => Some("even-horizontal".to_string()),
        3 | 4 => Some("tiled".to_string()),
        _ => Some("tiled".to_string()),
    }
}

/// Helper to convert window/pane coordinates to tmux target format
pub fn pane_target(session_name: &str, window_idx: usize, pane_idx: usize) -> String {
    format!("{}:{}.{}", session_name, window_idx, pane_idx)
}
