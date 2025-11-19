/// Embedded webview dashboard for Colony
///
/// This module provides a lightweight native window with full web capabilities
/// for visualizing colony data, creating tasks, and managing agents.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use wry::{
    application::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
    webview::WebViewBuilder,
};

use crate::colony::state::ColonyState;
use crate::types::{Agent, Task};

/// Launch the Colony dashboard in a webview window
pub fn show_dashboard() -> Result<()> {
    // Load colony state
    let state = ColonyState::load()
        .context("Failed to load colony state. Make sure colony is initialized.")?;

    show_dashboard_with_state(state)
}

/// Launch dashboard with provided state (for testing)
pub fn show_dashboard_with_state(state: ColonyState) -> Result<()> {
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("🐝 Colony Dashboard")
        .with_inner_size(wry::application::dpi::LogicalSize::new(1200.0, 800.0))
        .build(&event_loop)
        .context("Failed to create window")?;

    // Wrap state in Arc<Mutex> for thread-safe sharing
    let state = Arc::new(Mutex::new(state));
    let state_clone = Arc::clone(&state);

    let html = include_str!("dashboard.html");

    let _webview = WebViewBuilder::new(window)
        .context("Failed to create webview builder")?
        .with_html(html)
        .context("Failed to load HTML")?
        // Custom protocol for loading colony data
        .with_custom_protocol("colony".into(), move |request| {
            handle_colony_protocol(&state_clone, request)
        })
        // IPC handler for commands from JavaScript
        .with_ipc_handler(move |_window, message| {
            handle_ipc_message(&state, &message);
        })
        .build()
        .context("Failed to build webview")?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

/// Handle custom colony:// protocol requests
fn handle_colony_protocol(
    state: &Arc<Mutex<ColonyState>>,
    request: &wry::http::Request<Vec<u8>>,
) -> Result<wry::http::Response<Vec<u8>>, wry::Error> {
    let uri = request.uri().to_string();

    let response_data = match uri.as_str() {
        "colony://tasks" => {
            // Load tasks from state
            let state = state.lock().unwrap();
            let tasks: Vec<Task> = state.get_all_tasks();
            serde_json::to_string(&tasks).unwrap_or_else(|_| "[]".to_string())
        }
        "colony://agents" => {
            // Load agents from state
            let state = state.lock().unwrap();
            let agents: Vec<Agent> = state.get_all_agents();
            serde_json::to_string(&agents).unwrap_or_else(|_| "[]".to_string())
        }
        "colony://stats" => {
            // Get statistics
            let state = state.lock().unwrap();
            let stats = DashboardStats::from_state(&state);
            serde_json::to_string(&stats).unwrap_or_else(|_| "{}".to_string())
        }
        _ => {
            return Ok(wry::http::Response::builder()
                .status(404)
                .body(Vec::new())
                .unwrap());
        }
    };

    Ok(wry::http::Response::builder()
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(response_data.into_bytes())
        .unwrap())
}

/// Handle IPC messages from JavaScript
fn handle_ipc_message(state: &Arc<Mutex<ColonyState>>, message: &str) {
    println!("Received IPC message: {}", message);

    // Parse command
    if let Some((cmd, args)) = message.split_once(':') {
        match cmd {
            "create_task" => {
                println!("Create task requested: {}", args);
                // TODO: Implement task creation
            }
            "start_agent" => {
                println!("Start agent requested: {}", args);
                // TODO: Implement agent start
            }
            "stop_agent" => {
                println!("Stop agent requested: {}", args);
                // TODO: Implement agent stop
            }
            "refresh" => {
                println!("Refresh requested");
                // State is already live, just log
            }
            _ => {
                eprintln!("Unknown command: {}", cmd);
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DashboardStats {
    total_tasks: usize,
    pending_tasks: usize,
    in_progress_tasks: usize,
    completed_tasks: usize,
    blocked_tasks: usize,
    total_agents: usize,
    running_agents: usize,
    idle_agents: usize,
    error_agents: usize,
}

impl DashboardStats {
    fn from_state(state: &ColonyState) -> Self {
        let tasks = state.get_all_tasks();
        let agents = state.get_all_agents();

        Self {
            total_tasks: tasks.len(),
            pending_tasks: tasks.iter().filter(|t| t.status == "pending").count(),
            in_progress_tasks: tasks
                .iter()
                .filter(|t| t.status == "in-progress")
                .count(),
            completed_tasks: tasks.iter().filter(|t| t.status == "completed").count(),
            blocked_tasks: tasks.iter().filter(|t| t.status == "blocked").count(),
            total_agents: agents.len(),
            running_agents: agents.iter().filter(|a| a.status == "running").count(),
            idle_agents: agents.iter().filter(|a| a.status == "stopped").count(),
            error_agents: agents.iter().filter(|a| a.status == "error").count(),
        }
    }
}
