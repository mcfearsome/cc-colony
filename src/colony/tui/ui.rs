use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table, Tabs},
    Frame,
};

use super::app::{App, Tab};
use crate::colony::agent::AgentStatus;
use crate::colony::tasks::TaskStatus;

/// Render the main UI
pub fn render(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Length(3), // Metrics
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Status bar
        ])
        .split(f.size());

    // Render tabs
    render_tabs(f, app, chunks[0]);

    // Render metrics
    render_metrics(f, app, chunks[1]);

    // Render main content based on current tab
    match app.current_tab {
        Tab::Agents => render_agents(f, app, chunks[2]),
        Tab::Tasks => render_tasks(f, app, chunks[2]),
        Tab::Messages => render_messages(f, app, chunks[2]),
        Tab::State => render_state(f, app, chunks[2]),
        Tab::Help => render_help(f, chunks[2]),
    }

    // Render status bar
    render_status_bar(f, app, chunks[3]);

    // Render dialog on top if active
    if app.active_dialog.is_some() {
        render_dialog(f, app);
    }
}

fn render_tabs(f: &mut Frame, app: &App, area: Rect) {
    let tab_titles = vec!["1: Agents", "2: Tasks", "3: Messages", "4: State", "5: Help"];

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Colony Orchestration"),
        )
        .select(app.current_tab.index())
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, area);
}

fn render_metrics(f: &mut Frame, app: &App, area: Rect) {
    use crate::colony::state::{TaskStatus as StateTaskStatus, WorkflowStatus};

    // Calculate agent metrics
    let agents_running = app.data.agents.iter().filter(|a| a.status == AgentStatus::Running).count();
    let agents_idle = app.data.agents.iter().filter(|a| a.status == AgentStatus::Idle).count();
    let agents_failed = app.data.agents.iter().filter(|a| a.status == AgentStatus::Failed).count();
    let agents_total = app.data.agents.len();

    // Calculate task metrics (from tasks tab)
    let tasks_pending = app.data.tasks.get(&TaskStatus::Pending).map(|v| v.len()).unwrap_or(0);
    let tasks_in_progress = app.data.tasks.get(&TaskStatus::InProgress).map(|v| v.len()).unwrap_or(0);
    let tasks_completed = app.data.tasks.get(&TaskStatus::Completed).map(|v| v.len()).unwrap_or(0);
    let tasks_total = app.data.tasks.values().map(|v| v.len()).sum::<usize>();

    // Calculate message count
    let messages_count = app.data.messages.len();

    let mut metrics_text = vec![
        Span::styled("Agents: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{} ", agents_total), Style::default().fg(Color::White)),
        Span::styled("(", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", agents_running), Style::default().fg(Color::Green)),
        Span::styled(" run", Style::default().fg(Color::DarkGray)),
        Span::styled(", ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", agents_idle), Style::default().fg(Color::Gray)),
        Span::styled(" idle", Style::default().fg(Color::DarkGray)),
    ];

    if agents_failed > 0 {
        metrics_text.extend(vec![
            Span::styled(", ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", agents_failed), Style::default().fg(Color::Red)),
            Span::styled(" fail", Style::default().fg(Color::DarkGray)),
        ]);
    }

    metrics_text.extend(vec![
        Span::styled(")", Style::default().fg(Color::DarkGray)),
        Span::raw("  │  "),
        Span::styled("Tasks: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{} ", tasks_total), Style::default().fg(Color::White)),
        Span::styled("(", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", tasks_pending), Style::default().fg(Color::Yellow)),
        Span::styled(" pend", Style::default().fg(Color::DarkGray)),
        Span::styled(", ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", tasks_in_progress), Style::default().fg(Color::Cyan)),
        Span::styled(" prog", Style::default().fg(Color::DarkGray)),
        Span::styled(", ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{}", tasks_completed), Style::default().fg(Color::Green)),
        Span::styled(" done", Style::default().fg(Color::DarkGray)),
        Span::styled(")", Style::default().fg(Color::DarkGray)),
        Span::raw("  │  "),
        Span::styled("Messages: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{}", messages_count), Style::default().fg(Color::White)),
    ]);

    // Add state metrics if enabled
    if app.data.state_enabled {
        let state_tasks_ready = app.data.state_tasks.iter().filter(|t| t.status == StateTaskStatus::Ready).count();
        let state_tasks_in_progress = app.data.state_tasks.iter().filter(|t| t.status == StateTaskStatus::InProgress).count();
        let state_tasks_completed = app.data.state_tasks.iter().filter(|t| t.status == StateTaskStatus::Completed).count();
        let state_tasks_blocked = app.data.state_tasks.iter().filter(|t| t.status == StateTaskStatus::Blocked).count();
        let state_tasks_total = app.data.state_tasks.len();

        let state_workflows_running = app.data.state_workflows.iter().filter(|w| w.status == WorkflowStatus::Running).count();
        let state_workflows_completed = app.data.state_workflows.iter().filter(|w| w.status == WorkflowStatus::Completed).count();
        let state_workflows_total = app.data.state_workflows.len();

        metrics_text.extend(vec![
            Span::raw("  │  "),
            Span::styled("State Tasks: ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{} ", state_tasks_total), Style::default().fg(Color::White)),
            Span::styled("(", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", state_tasks_ready), Style::default().fg(Color::Green)),
            Span::styled(" rdy", Style::default().fg(Color::DarkGray)),
            Span::styled(", ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", state_tasks_in_progress), Style::default().fg(Color::Cyan)),
            Span::styled(" prog", Style::default().fg(Color::DarkGray)),
        ]);

        if state_tasks_blocked > 0 {
            metrics_text.extend(vec![
                Span::styled(", ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{}", state_tasks_blocked), Style::default().fg(Color::Red)),
                Span::styled(" blk", Style::default().fg(Color::DarkGray)),
            ]);
        }

        metrics_text.extend(vec![
            Span::styled(")", Style::default().fg(Color::DarkGray)),
            Span::raw("  │  "),
            Span::styled("Workflows: ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{} ", state_workflows_total), Style::default().fg(Color::White)),
            Span::styled("(", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", state_workflows_running), Style::default().fg(Color::Cyan)),
            Span::styled(" run", Style::default().fg(Color::DarkGray)),
            Span::styled(", ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", state_workflows_completed), Style::default().fg(Color::Green)),
            Span::styled(" done", Style::default().fg(Color::DarkGray)),
            Span::styled(")", Style::default().fg(Color::DarkGray)),
        ]);
    }

    let metrics_line = vec![Line::from(metrics_text)];

    let paragraph = Paragraph::new(metrics_line)
        .block(Block::default().borders(Borders::ALL).title("Metrics"))
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, area);
}

fn render_agents(f: &mut Frame, app: &App, area: Rect) {
    let agents = &app.data.agents;

    // Create table rows
    let rows: Vec<Row> = agents
        .iter()
        .map(|agent| {
            let status_style = match agent.status {
                AgentStatus::Running => Style::default().fg(Color::Green),
                AgentStatus::Idle => Style::default().fg(Color::Gray),
                AgentStatus::Completed => Style::default().fg(Color::Blue),
                AgentStatus::Failed => Style::default().fg(Color::Red),
            };

            let status_text = format!("{:?}", agent.status).to_uppercase();
            let pid_text = agent
                .pid
                .map(|p| p.to_string())
                .unwrap_or_else(|| "-".to_string());
            let task_text = agent
                .current_task
                .as_ref()
                .map(|t| truncate(t, 30))
                .unwrap_or_else(|| "-".to_string());

            Row::new(vec![
                agent.id.clone(),
                truncate(&agent.role, 25),
                status_text,
                pid_text,
                task_text,
            ])
            .style(status_style)
        })
        .collect();

    let table = Table::new(
        rows,
        vec![
            Constraint::Length(20), // Agent ID
            Constraint::Length(25), // Role
            Constraint::Length(12), // Status
            Constraint::Length(10), // PID
            Constraint::Min(30),    // Current Task
        ],
    )
    .header(
        Row::new(vec!["Agent ID", "Role", "Status", "PID", "Current Task"])
            .style(Style::default().add_modifier(Modifier::BOLD))
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Agents ({} total)", agents.len())),
    );

    f.render_widget(table, area);
}

fn render_tasks(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Summary
            Constraint::Min(0),    // Task list
        ])
        .split(area);

    // Render summary
    render_task_summary(f, app, chunks[0]);

    // Render task list
    render_task_list(f, app, chunks[1]);
}

fn render_task_summary(f: &mut Frame, app: &App, area: Rect) {
    let counts = app.data.task_counts();
    let total = app.data.total_tasks();
    let completion_pct = app.data.completion_percentage();

    let pending = counts.get(&TaskStatus::Pending).copied().unwrap_or(0);
    let claimed = counts.get(&TaskStatus::Claimed).copied().unwrap_or(0);
    let in_progress = counts.get(&TaskStatus::InProgress).copied().unwrap_or(0);
    let blocked = counts.get(&TaskStatus::Blocked).copied().unwrap_or(0);
    let completed = counts.get(&TaskStatus::Completed).copied().unwrap_or(0);

    let summary_text = vec![
        Line::from(vec![
            Span::styled("⏳ Pending: ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}  ", pending)),
            Span::styled("👤 Claimed: ", Style::default().fg(Color::Cyan)),
            Span::raw(format!("{}  ", claimed)),
            Span::styled("🔄 In Progress: ", Style::default().fg(Color::Blue)),
            Span::raw(format!("{}  ", in_progress)),
            Span::styled("🚫 Blocked: ", Style::default().fg(Color::Red)),
            Span::raw(format!("{}  ", blocked)),
            Span::styled("✅ Completed: ", Style::default().fg(Color::Green)),
            Span::raw(completed.to_string()),
        ]),
        Line::from(""),
        Line::from(render_progress_bar(completion_pct)),
    ];

    let paragraph = Paragraph::new(summary_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Task Summary (Total: {})", total)),
    );

    f.render_widget(paragraph, area);
}

fn render_task_list(f: &mut Frame, app: &App, area: Rect) {
    let mut items = Vec::new();

    // Show tasks by status
    let status_order = vec![
        (TaskStatus::InProgress, Color::Blue),
        (TaskStatus::Blocked, Color::Red),
        (TaskStatus::Claimed, Color::Cyan),
        (TaskStatus::Pending, Color::Yellow),
        (TaskStatus::Completed, Color::Green),
    ];

    for (status, color) in status_order {
        if let Some(tasks) = app.data.tasks.get(&status) {
            if !tasks.is_empty() {
                // Add status header
                items.push(ListItem::new(Line::from(vec![Span::styled(
                    format!(
                        "\n{} {} ({})",
                        status.emoji(),
                        status.display(),
                        tasks.len()
                    ),
                    Style::default()
                        .fg(color)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                )])));

                // Add tasks
                for task in tasks {
                    let assigned = task
                        .claimed_by
                        .as_ref()
                        .map(|id| format!(" [{}]", id))
                        .unwrap_or_default();

                    let progress = if task.progress > 0 {
                        format!(" {}%", task.progress)
                    } else {
                        String::new()
                    };

                    items.push(ListItem::new(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(&task.title, Style::default().fg(Color::White)),
                        Span::styled(assigned, Style::default().fg(Color::Gray)),
                        Span::styled(progress, Style::default().fg(Color::Cyan)),
                    ])));
                }
            }
        }
    }

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Task List"))
        .style(Style::default().fg(Color::White));

    f.render_widget(list, area);
}

fn render_messages(f: &mut Frame, app: &App, area: Rect) {
    let messages = &app.data.messages;

    let items: Vec<ListItem> = messages
        .iter()
        .map(|msg| {
            let time = msg
                .timestamp
                .split('T')
                .nth(1)
                .and_then(|t| t.split('.').next())
                .unwrap_or("??:??:??");

            let to_display = if msg.to == "all" {
                "[BROADCAST]".to_string()
            } else {
                format!("→ {}", msg.to)
            };

            let line = Line::from(vec![
                Span::styled(time, Style::default().fg(Color::Gray)),
                Span::raw(" "),
                Span::styled(&msg.from, Style::default().fg(Color::Cyan)),
                Span::raw(" "),
                Span::styled(to_display, Style::default().fg(Color::Yellow)),
                Span::raw(": "),
                Span::styled(
                    truncate(&msg.content, 80),
                    Style::default().fg(Color::White),
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Messages (showing {} most recent)", messages.len())),
    );

    f.render_widget(list, area);
}

fn render_state(f: &mut Frame, app: &App, area: Rect) {
    use crate::colony::state::{TaskStatus as StateTaskStatus, WorkflowStatus};

    if !app.data.state_enabled {
        let text = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "Shared State Not Configured",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from("  Shared state is not enabled in colony.yml"),
            Line::from(""),
            Line::from("  To enable shared state, add the following to colony.yml:"),
            Line::from(""),
            Line::from("  shared_state:"),
            Line::from("    backend: git-backed"),
            Line::from("    location: in-repo"),
            Line::from(""),
            Line::from("  Then restart colony with 'colony start'"),
        ];

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Shared State"))
            .style(Style::default().fg(Color::White));

        f.render_widget(paragraph, area);
        return;
    }

    // Split area into tasks and workflows sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    // Render tasks
    let task_rows: Vec<Row> = app
        .data
        .state_tasks
        .iter()
        .map(|task| {
            let status_style = match task.status {
                StateTaskStatus::Ready => Style::default().fg(Color::Green),
                StateTaskStatus::Blocked => Style::default().fg(Color::Red),
                StateTaskStatus::InProgress => Style::default().fg(Color::Cyan),
                StateTaskStatus::Completed => Style::default().fg(Color::Blue),
                StateTaskStatus::Cancelled => Style::default().fg(Color::Gray),
            };

            let status_icon = match task.status {
                StateTaskStatus::Ready => "●",
                StateTaskStatus::Blocked => "◆",
                StateTaskStatus::InProgress => "◐",
                StateTaskStatus::Completed => "✓",
                StateTaskStatus::Cancelled => "✗",
            };

            let assigned = task.assigned.as_deref().unwrap_or("-");
            let blockers = if task.blockers.is_empty() {
                "-".to_string()
            } else {
                task.blockers.len().to_string()
            };

            Row::new(vec![
                status_icon.to_string(),
                truncate(&task.id, 12),
                truncate(&task.title, 35),
                truncate(assigned, 15),
                blockers,
            ])
            .style(status_style)
        })
        .collect();

    let tasks_table = Table::new(
        task_rows,
        vec![
            Constraint::Length(3),  // Status icon
            Constraint::Length(12), // ID
            Constraint::Length(35), // Title
            Constraint::Length(15), // Assigned
            Constraint::Length(8),  // Blockers
        ],
    )
    .header(
        Row::new(vec!["", "ID", "Title", "Assigned", "Blocks"])
            .style(Style::default().add_modifier(Modifier::BOLD))
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Tasks ({} total)", app.data.state_tasks.len())),
    );

    f.render_widget(tasks_table, chunks[0]);

    // Render workflows
    let workflow_rows: Vec<Row> = app
        .data
        .state_workflows
        .iter()
        .map(|workflow| {
            let status_style = match workflow.status {
                WorkflowStatus::Pending => Style::default().fg(Color::Yellow),
                WorkflowStatus::Running => Style::default().fg(Color::Cyan),
                WorkflowStatus::Completed => Style::default().fg(Color::Green),
                WorkflowStatus::Failed => Style::default().fg(Color::Red),
                WorkflowStatus::Cancelled => Style::default().fg(Color::Gray),
            };

            let status_icon = match workflow.status {
                WorkflowStatus::Pending => "○",
                WorkflowStatus::Running => "◐",
                WorkflowStatus::Completed => "✓",
                WorkflowStatus::Failed => "✗",
                WorkflowStatus::Cancelled => "✗",
            };

            let step = workflow.current_step.as_deref().unwrap_or("-");

            Row::new(vec![
                status_icon.to_string(),
                truncate(&workflow.id, 12),
                truncate(&workflow.name, 40),
                truncate(step, 20),
            ])
            .style(status_style)
        })
        .collect();

    let workflows_table = Table::new(
        workflow_rows,
        vec![
            Constraint::Length(3),  // Status icon
            Constraint::Length(12), // ID
            Constraint::Length(40), // Name
            Constraint::Length(20), // Current step
        ],
    )
    .header(
        Row::new(vec!["", "ID", "Name", "Current Step"])
            .style(Style::default().add_modifier(Modifier::BOLD))
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Workflows ({} total)", app.data.state_workflows.len())),
    );

    f.render_widget(workflows_table, chunks[1]);
}

fn render_help(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Keyboard Shortcuts",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("  Navigation:"),
        Line::from("    1-5, Tab       Switch between tabs"),
        Line::from("    ↑/↓, j/k       Scroll up/down (coming soon)"),
        Line::from("    PgUp/PgDn      Page up/down (coming soon)"),
        Line::from(""),
        Line::from("  Actions:"),
        Line::from("    r              Refresh data"),
        Line::from("    b              Broadcast message (coming soon)"),
        Line::from("    t              Create task (coming soon)"),
        Line::from("    m              Send message to agent (coming soon)"),
        Line::from("    ?              Show this help"),
        Line::from(""),
        Line::from("  General:"),
        Line::from("    q, Ctrl+C      Quit"),
        Line::from("    Esc            Cancel/Go back"),
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Tabs Overview",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("  1: Agents      - View running agents and their current tasks"),
        Line::from("  2: Tasks       - Monitor task queue (pending, in progress, completed)"),
        Line::from("  3: Messages    - See message flow between agents and colony"),
        Line::from("  4: State       - Git-backed shared state (tasks, workflows)"),
        Line::from("  5: Help        - This help screen"),
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Metrics Panel",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("  The metrics panel (always visible) shows real-time statistics:"),
        Line::from("    • Agent counts by status (running, idle, failed)"),
        Line::from("    • Task counts by status (pending, in progress, completed)"),
        Line::from("    • Pending message count"),
        Line::from("    • State tasks and workflows (when shared state enabled)"),
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            "CLI Commands",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("  Run these commands outside the TUI:"),
        Line::from("    colony health       - Comprehensive system health diagnostics"),
        Line::from("    colony state task   - Manage shared state tasks"),
        Line::from("    colony state workflow - Manage workflows"),
        Line::from("    colony status       - Quick colony status overview"),
        Line::from(""),
        Line::from(""),
        Line::from(vec![Span::styled(
            "About",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("  Colony TUI provides real-time monitoring and orchestration for"),
        Line::from("  multi-agent Claude Code systems. View agent activity, coordinate"),
        Line::from("  tasks, monitor messages, and track distributed workflows."),
        Line::from(""),
        Line::from("  Data refreshes automatically every 2 seconds. Press 'r' to force refresh."),
        Line::from(""),
        Line::from("  Shared state (Tab 4) enables git-backed coordination across sessions."),
        Line::from("  Configure in colony.yml to enable distributed task management."),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .style(Style::default().fg(Color::White));

    f.render_widget(paragraph, area);
}

fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let running_count = app
        .data
        .agents
        .iter()
        .filter(|a| a.status == AgentStatus::Running)
        .count();

    let status_text = if let Some((ref message, is_error)) = app.status_message {
        // Show status message
        vec![Line::from(vec![
            Span::styled(
                if is_error { "Error: " } else { "Status: " },
                Style::default()
                    .fg(if is_error { Color::Red } else { Color::Green })
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                message,
                Style::default().fg(if is_error { Color::Red } else { Color::White }),
            ),
        ])]
    } else {
        // Show normal status
        vec![Line::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{} agents running", running_count),
                Style::default().fg(Color::Green),
            ),
            Span::raw("  |  "),
            Span::styled("Shortcuts: ", Style::default().fg(Color::Gray)),
            Span::raw("q=Quit  r=Refresh  b=Broadcast  t=Task  m=Message  ?=Help"),
        ])]
    };

    let paragraph = Paragraph::new(status_text).block(Block::default().borders(Borders::ALL));

    f.render_widget(paragraph, area);
}

fn render_progress_bar(percentage: u8) -> Vec<Span<'static>> {
    let bar_width = 50;
    let filled = (percentage as usize * bar_width) / 100;
    let empty = bar_width - filled;

    let mut spans = vec![Span::raw("  Progress: [")];

    if filled > 0 {
        spans.push(Span::styled(
            "█".repeat(filled),
            Style::default().fg(Color::Green),
        ));
    }

    if empty > 0 {
        spans.push(Span::styled(
            "░".repeat(empty),
            Style::default().fg(Color::DarkGray),
        ));
    }

    spans.push(Span::raw(format!("] {}%", percentage)));

    spans
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Render a dialog overlay
fn render_dialog(f: &mut Frame, app: &App) {
    use super::app::Dialog;

    let dialog = match &app.active_dialog {
        Some(d) => d,
        None => return,
    };

    // Create a centered popup area
    let area = f.size();
    let width = std::cmp::min(80, area.width - 4);
    let height = 10;
    let x = (area.width.saturating_sub(width)) / 2;
    let y = (area.height.saturating_sub(height)) / 2;

    let popup_area = Rect {
        x,
        y,
        width,
        height,
    };

    // Clear the area behind the dialog
    let clear_block = Block::default()
        .borders(Borders::NONE)
        .style(Style::default().bg(Color::Black));
    f.render_widget(clear_block, popup_area);

    // Create dialog content
    let mut lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            dialog.prompt(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
    ];

    // Show previous inputs for multi-step dialogs
    match dialog {
        Dialog::CreateTask { step } | Dialog::SendMessage { step } => {
            if *step > 0 {
                lines.push(Line::from(vec![Span::styled(
                    "Previous inputs:",
                    Style::default().fg(Color::Gray),
                )]));

                for (i, input) in app.dialog_inputs.iter().enumerate() {
                    let temp_dialog = if matches!(dialog, Dialog::CreateTask { .. }) {
                        Dialog::CreateTask { step: i }
                    } else {
                        Dialog::SendMessage { step: i }
                    };
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("  {}: ", temp_dialog.prompt()),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::styled(input, Style::default().fg(Color::Gray)),
                    ]));
                }
                lines.push(Line::from(""));
            }

            // Show progress
            let progress = format!("Step {} of {}", step + 1, dialog.total_steps());
            lines.insert(
                0,
                Line::from(vec![Span::styled(
                    progress,
                    Style::default().fg(Color::Yellow),
                )]),
            );
        }
        _ => {}
    }

    // Show current input with cursor
    let input_line = vec![
        Span::styled("> ", Style::default().fg(Color::Green)),
        Span::styled(&app.input_buffer, Style::default().fg(Color::White)),
        Span::styled("█", Style::default().fg(Color::White)),
    ];
    lines.push(Line::from(input_line));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::styled(" to confirm  ", Style::default().fg(Color::Gray)),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::styled(" to cancel", Style::default().fg(Color::Gray)),
    ]));

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(dialog.title())
                .title_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .style(Style::default().bg(Color::Black));

    f.render_widget(paragraph, popup_area);
}
