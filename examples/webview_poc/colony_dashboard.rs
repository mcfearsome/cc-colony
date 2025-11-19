// Full-featured Colony dashboard using embedded webview
// This demonstrates bi-directional communication between Rust and JavaScript

use serde::{Deserialize, Serialize};
use wry::{
    application::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
    webview::{WebView, WebViewBuilder},
};

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    id: String,
    title: String,
    status: String,
    assigned_to: String,
    priority: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Agent {
    id: String,
    role: String,
    status: String,
    current_task: Option<String>,
}

fn main() -> wry::Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("🐝 Colony Dashboard")
        .with_inner_size(wry::application::dpi::LogicalSize::new(1200, 800))
        .build(&event_loop)
        .unwrap();

    // Load the HTML dashboard
    let html = include_str!("dashboard.html");

    let webview = WebViewBuilder::new(window)?
        .with_html(html)?
        // Enable dev tools for debugging
        .with_devtools(true)
        // Custom protocol for loading local resources
        .with_custom_protocol("colony".into(), move |request| {
            // Handle colony:// URLs for loading local resources
            let uri = request.uri();
            if uri == "colony://tasks" {
                // In real implementation, load from colony data
                let tasks = get_mock_tasks();
                let json = serde_json::to_string(&tasks).unwrap();

                Ok(wry::http::Response::builder()
                    .header("Content-Type", "application/json")
                    .body(json.into_bytes().into())
                    .unwrap())
            } else if uri == "colony://agents" {
                let agents = get_mock_agents();
                let json = serde_json::to_string(&agents).unwrap();

                Ok(wry::http::Response::builder()
                    .header("Content-Type", "application/json")
                    .body(json.into_bytes().into())
                    .unwrap())
            } else {
                Ok(wry::http::Response::builder()
                    .status(404)
                    .body(Vec::new().into())
                    .unwrap())
            }
        })
        // IPC handler for JavaScript -> Rust communication
        .with_ipc_handler(|_window: &WebView, message: String| {
            println!("Received IPC message from JS: {}", message);

            // Parse command
            if message.starts_with("create_task:") {
                println!("Creating task...");
                // In real implementation, call colony's task creation
            } else if message.starts_with("start_agent:") {
                println!("Starting agent...");
            }
        })
        .build()?;

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

fn get_mock_tasks() -> Vec<Task> {
    vec![
        Task {
            id: "task-1".to_string(),
            title: "Setup Project".to_string(),
            status: "completed".to_string(),
            assigned_to: "backend-1".to_string(),
            priority: "high".to_string(),
        },
        Task {
            id: "task-2".to_string(),
            title: "Implement Auth".to_string(),
            status: "in-progress".to_string(),
            assigned_to: "backend-1".to_string(),
            priority: "critical".to_string(),
        },
        Task {
            id: "task-3".to_string(),
            title: "Build UI".to_string(),
            status: "pending".to_string(),
            assigned_to: "frontend-1".to_string(),
            priority: "medium".to_string(),
        },
    ]
}

fn get_mock_agents() -> Vec<Agent> {
    vec![
        Agent {
            id: "backend-1".to_string(),
            role: "Backend Engineer".to_string(),
            status: "running".to_string(),
            current_task: Some("task-2".to_string()),
        },
        Agent {
            id: "frontend-1".to_string(),
            role: "Frontend Engineer".to_string(),
            status: "idle".to_string(),
            current_task: None,
        },
    ]
}
