// Example: Native GUI widget using egui
// Lightweight alternative to browser-based UI

#[cfg(feature = "gui")]
use eframe::egui;

#[cfg(feature = "gui")]
use egui_plot::{Bar, BarChart, Line, Plot, PlotPoints};

#[derive(Debug, Clone)]
pub struct Task {
    id: String,
    title: String,
    status: TaskStatus,
    assigned_to: String,
    priority: Priority,
    description: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Blocked,
    Completed,
}

#[derive(Debug, Clone)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[cfg(feature = "gui")]
pub struct ColonyDashboard {
    tasks: Vec<Task>,
    selected_task: Option<usize>,
    agent_cpu_history: Vec<(String, Vec<f64>)>,
    filter_status: Option<TaskStatus>,
}

#[cfg(feature = "gui")]
impl Default for ColonyDashboard {
    fn default() -> Self {
        Self {
            tasks: create_demo_tasks(),
            selected_task: None,
            agent_cpu_history: vec![
                ("backend-1".to_string(), vec![10.0, 20.0, 35.0, 45.0, 50.0, 48.0, 45.0]),
                ("frontend-1".to_string(), vec![5.0, 8.0, 15.0, 25.0, 32.0, 35.0, 34.0]),
                ("test-1".to_string(), vec![60.0, 65.0, 70.0, 75.0, 78.0, 77.0, 76.0]),
            ],
            filter_status: None,
        }
    }
}

#[cfg(feature = "gui")]
impl eframe::App for ColonyDashboard {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top menu bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🐝 Colony Dashboard");
                ui.separator();

                ui.label("Filter:");
                if ui.selectable_label(self.filter_status.is_none(), "All").clicked() {
                    self.filter_status = None;
                }
                if ui.selectable_label(
                    self.filter_status == Some(TaskStatus::Pending),
                    "Pending"
                ).clicked() {
                    self.filter_status = Some(TaskStatus::Pending);
                }
                if ui.selectable_label(
                    self.filter_status == Some(TaskStatus::InProgress),
                    "In Progress"
                ).clicked() {
                    self.filter_status = Some(TaskStatus::InProgress);
                }
                if ui.selectable_label(
                    self.filter_status == Some(TaskStatus::Blocked),
                    "Blocked"
                ).clicked() {
                    self.filter_status = Some(TaskStatus::Blocked);
                }
            });
        });

        // Left panel - Task list
        egui::SidePanel::left("task_list")
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("Tasks");
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let filtered_tasks: Vec<_> = self.tasks.iter().enumerate()
                        .filter(|(_, t)| {
                            self.filter_status.as_ref()
                                .map(|f| &t.status == f)
                                .unwrap_or(true)
                        })
                        .collect();

                    for (idx, task) in filtered_tasks {
                        let is_selected = self.selected_task == Some(idx);

                        let color = match task.status {
                            TaskStatus::Pending => egui::Color32::from_rgb(220, 220, 170),
                            TaskStatus::InProgress => egui::Color32::from_rgb(78, 201, 176),
                            TaskStatus::Blocked => egui::Color32::from_rgb(244, 135, 113),
                            TaskStatus::Completed => egui::Color32::from_rgb(96, 139, 78),
                        };

                        ui.horizontal(|ui| {
                            ui.colored_label(color, "●");

                            if ui.selectable_label(is_selected, &task.title).clicked() {
                                self.selected_task = Some(idx);
                            }
                        });
                    }
                });
            });

        // Main panel
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(idx) = self.selected_task {
                if let Some(task) = self.tasks.get(idx) {
                    // Task details
                    ui.heading(&task.title);
                    ui.separator();

                    ui.horizontal(|ui| {
                        ui.label("Status:");
                        let color = match task.status {
                            TaskStatus::Pending => egui::Color32::YELLOW,
                            TaskStatus::InProgress => egui::Color32::GREEN,
                            TaskStatus::Blocked => egui::Color32::RED,
                            TaskStatus::Completed => egui::Color32::GRAY,
                        };
                        ui.colored_label(color, format!("{:?}", task.status));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Priority:");
                        let color = match task.priority {
                            Priority::Critical => egui::Color32::RED,
                            Priority::High => egui::Color32::from_rgb(255, 165, 0),
                            Priority::Medium => egui::Color32::YELLOW,
                            Priority::Low => egui::Color32::GREEN,
                        };
                        ui.colored_label(color, format!("{:?}", task.priority));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Assigned to:");
                        ui.strong(&task.assigned_to);
                    });

                    ui.add_space(10.0);
                    ui.label("Description:");
                    ui.label(&task.description);

                    ui.add_space(10.0);
                    ui.separator();

                    // Action buttons
                    ui.horizontal(|ui| {
                        if ui.button("Start Task").clicked() {
                            println!("Starting task: {}", task.id);
                        }
                        if ui.button("Block Task").clicked() {
                            println!("Blocking task: {}", task.id);
                        }
                        if ui.button("Complete Task").clicked() {
                            println!("Completing task: {}", task.id);
                        }
                    });
                }
            } else {
                ui.vertical_centered(|ui| {
                    ui.add_space(100.0);
                    ui.heading("Select a task to view details");
                });
            }

            ui.add_space(20.0);
            ui.separator();

            // Task status chart
            ui.heading("Task Distribution");
            let pending = self.tasks.iter().filter(|t| matches!(t.status, TaskStatus::Pending)).count() as f64;
            let in_progress = self.tasks.iter().filter(|t| matches!(t.status, TaskStatus::InProgress)).count() as f64;
            let blocked = self.tasks.iter().filter(|t| matches!(t.status, TaskStatus::Blocked)).count() as f64;
            let completed = self.tasks.iter().filter(|t| matches!(t.status, TaskStatus::Completed)).count() as f64;

            Plot::new("task_distribution")
                .view_aspect(2.0)
                .show(ui, |plot_ui| {
                    plot_ui.bar_chart(
                        BarChart::new(vec![
                            Bar::new(0.0, pending).fill(egui::Color32::from_rgb(220, 220, 170)),
                            Bar::new(1.0, in_progress).fill(egui::Color32::from_rgb(78, 201, 176)),
                            Bar::new(2.0, blocked).fill(egui::Color32::from_rgb(244, 135, 113)),
                            Bar::new(3.0, completed).fill(egui::Color32::from_rgb(96, 139, 78)),
                        ])
                        .width(0.7)
                    );
                });

            ui.add_space(20.0);

            // CPU usage chart
            ui.heading("Agent CPU Usage");
            Plot::new("cpu_usage")
                .view_aspect(2.0)
                .legend(Default::default())
                .show(ui, |plot_ui| {
                    for (agent, history) in &self.agent_cpu_history {
                        let points: PlotPoints = history.iter()
                            .enumerate()
                            .map(|(i, v)| [i as f64, *v])
                            .collect();

                        plot_ui.line(Line::new(points).name(agent));
                    }
                });
        });

        // Request repaint for live updates
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }
}

#[cfg(feature = "gui")]
fn create_demo_tasks() -> Vec<Task> {
    vec![
        Task {
            id: "task-1".to_string(),
            title: "Setup Project Structure".to_string(),
            status: TaskStatus::Completed,
            assigned_to: "backend-1".to_string(),
            priority: Priority::High,
            description: "Initialize project with proper directory structure and configuration.".to_string(),
        },
        Task {
            id: "task-2".to_string(),
            title: "Implement Authentication".to_string(),
            status: TaskStatus::InProgress,
            assigned_to: "backend-1".to_string(),
            priority: Priority::Critical,
            description: "Add OAuth2 authentication with PKCE flow for secure user login.".to_string(),
        },
        Task {
            id: "task-3".to_string(),
            title: "Build Frontend UI".to_string(),
            status: TaskStatus::Pending,
            assigned_to: "frontend-1".to_string(),
            priority: Priority::High,
            description: "Create responsive UI components using React and Tailwind CSS.".to_string(),
        },
        Task {
            id: "task-4".to_string(),
            title: "Write Integration Tests".to_string(),
            status: TaskStatus::Pending,
            assigned_to: "test-1".to_string(),
            priority: Priority::Medium,
            description: "Develop comprehensive integration tests for all API endpoints.".to_string(),
        },
        Task {
            id: "task-5".to_string(),
            title: "Deploy to Production".to_string(),
            status: TaskStatus::Blocked,
            assigned_to: "devops-1".to_string(),
            priority: Priority::Critical,
            description: "Setup CI/CD pipeline and deploy to production environment.".to_string(),
        },
    ]
}

#[cfg(feature = "gui")]
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Colony Dashboard",
        options,
        Box::new(|_cc| Box::new(ColonyDashboard::default())),
    )
}

#[cfg(not(feature = "gui"))]
fn main() {
    eprintln!("This example requires the 'gui' feature.");
    eprintln!("Run with: cargo run --bin egui_widget --features gui");
}
