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
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Status bar
        ])
        .split(f.size());

    // Render tabs
    render_tabs(f, app, chunks[0]);

    // Render main content based on current tab
    match app.current_tab {
        Tab::Agents => render_agents(f, app, chunks[1]),
        Tab::Tasks => render_tasks(f, app, chunks[1]),
        Tab::Messages => render_messages(f, app, chunks[1]),
        Tab::Help => render_help(f, chunks[1]),
    }

    // Render status bar
    render_status_bar(f, app, chunks[2]);
}

fn render_tabs(f: &mut Frame, app: &App, area: Rect) {
    let tab_titles = vec!["1: Agents", "2: Tasks", "3: Messages", "4: Help"];

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
        Line::from("    1-4, Tab       Switch between tabs"),
        Line::from("    ↑/↓, j/k       Scroll up/down"),
        Line::from("    PgUp/PgDn      Page up/down"),
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
            "About",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from("  This TUI provides real-time monitoring and interaction with the colony"),
        Line::from("  orchestration system. Use it to view agent status, track tasks,"),
        Line::from("  monitor messages, and interact with the colony."),
        Line::from(""),
        Line::from("  Data refreshes automatically every 2 seconds. Press 'r' to force refresh."),
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

    let status_text = vec![Line::from(vec![
        Span::styled("Status: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{} agents running", running_count),
            Style::default().fg(Color::Green),
        ),
        Span::raw("  |  "),
        Span::styled("Shortcuts: ", Style::default().fg(Color::Gray)),
        Span::raw("q=Quit  r=Refresh  ?=Help  1-4=Switch Tab"),
    ])];

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
