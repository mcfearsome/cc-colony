# Lightweight UI Rendering Options for cc-colony

**Goal**: Rich widget-like views without launching a full browser

---

## Option 1: Terminal Graphics Protocols ⭐ RECOMMENDED

Modern terminals support rich graphics directly:

### Kitty Graphics Protocol
```rust
// src/colony/ui/terminal_graphics.rs
use std::fs::File;
use std::io::Write;
use base64::{Engine as _, engine::general_purpose};

pub fn render_graph_to_terminal(graph_data: &str) -> Result<()> {
    // Generate chart using plotters
    let chart = generate_chart(graph_data)?;

    // Encode as base64
    let encoded = general_purpose::STANDARD.encode(&chart);

    // Use Kitty graphics protocol
    print!("\x1b_Gf=100,a=T,m=1;{}\x1b\\", encoded);

    Ok(())
}
```

**Supported terminals**: Kitty, WezTerm, Konsole
**Pros**: Zero dependencies, works in terminal, fast
**Cons**: Terminal-dependent

### Sixel Graphics
```rust
use viuer::{Config, print_from_file};

pub fn show_image_in_terminal(path: &str) -> Result<()> {
    let conf = Config {
        transparent: true,
        absolute_offset: false,
        ..Default::default()
    };

    print_from_file(path, &conf)?;
    Ok(())
}
```

**Supported terminals**: xterm, mlterm, foot, iTerm2 (with imgcat)
**Library**: `viuer` crate
**Pros**: Wide terminal support, simple
**Cons**: Image-based (not interactive)

---

## Option 2: Plotters (Terminal Charts) ⭐⭐ HIGHLY RECOMMENDED

Pure Rust charting library that renders to terminal, PNG, or SVG:

```rust
use plotters::prelude::*;
use plotters::backend::BitMapBackend;

pub fn render_task_timeline() -> Result<()> {
    let root = BitMapBackend::new("task-timeline.png", (800, 600))
        .into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Task Timeline", ("sans-serif", 50))
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_2d(0f32..10f32, 0f32..10f32)?;

    chart.configure_mesh().draw()?;

    // Draw task bars
    chart.draw_series(
        tasks.iter().map(|task| {
            Rectangle::new([
                (task.start, task.agent_id),
                (task.end, task.agent_id + 1.0)
            ], &BLUE.mix(0.5))
        })
    )?;

    root.present()?;

    // Display in terminal using terminal graphics protocol
    show_image_in_terminal("task-timeline.png")?;

    Ok(())
}
```

**With ASCII Backend** (works everywhere):
```rust
use plotters::prelude::*;
use plotters::backend::text::TextBackend;

pub fn render_ascii_chart() -> Result<()> {
    let mut buffer = String::new();
    {
        let root = TextBackend::new(&mut buffer, (80, 30))
            .into_drawing_area();

        let mut chart = ChartBuilder::on(&root)
            .caption("CPU Usage", ('x', 2))
            .build_cartesian_2d(0..100, 0..100)?;

        chart.draw_series(
            LineSeries::new(
                cpu_data.iter().enumerate()
                    .map(|(x, y)| (x as i32, *y)),
                &RED
            )
        )?;
    }

    println!("{}", buffer);
    Ok(())
}
```

**Output**:
```
       CPU Usage
100 ┤                  ╭─
 90 ┤                ╭─╯
 80 ┤              ╭─╯
 70 ┤            ╭─╯
 60 ┤          ╭─╯
 50 ┤        ╭─╯
 40 ┤      ╭─╯
 30 ┤    ╭─╯
 20 ┤  ╭─╯
 10 ┤╭─╯
  0 ┼──────────────────────
    0    20   40   60   80
```

---

## Option 3: egui (Immediate Mode GUI) ⭐⭐⭐ BEST FOR RICH UIs

Lightweight native GUI framework, ~2MB memory overhead:

```rust
// Cargo.toml
[dependencies]
eframe = "0.27"
egui = "0.27"
egui_plot = "0.27"

// src/colony/ui/widgets/task_board.rs
use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};

pub struct TaskBoardWidget {
    tasks: Vec<Task>,
    selected: Option<String>,
}

impl eframe::App for TaskBoardWidget {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🐝 Colony Task Board");

            // Task list
            egui::ScrollArea::vertical().show(ui, |ui| {
                for task in &self.tasks {
                    let color = match task.status {
                        Status::Pending => egui::Color32::YELLOW,
                        Status::InProgress => egui::Color32::GREEN,
                        Status::Blocked => egui::Color32::RED,
                        Status::Completed => egui::Color32::GRAY,
                    };

                    ui.horizontal(|ui| {
                        ui.colored_label(color, "●");
                        if ui.selectable_label(
                            self.selected.as_ref() == Some(&task.id),
                            &task.title
                        ).clicked() {
                            self.selected = Some(task.id.clone());
                        }
                    });
                }
            });

            ui.separator();

            // Task details
            if let Some(task_id) = &self.selected {
                if let Some(task) = self.tasks.iter().find(|t| &t.id == task_id) {
                    ui.heading(&task.title);
                    ui.label(&task.description);

                    ui.horizontal(|ui| {
                        ui.label("Assigned to:");
                        ui.strong(&task.assigned_to);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Priority:");
                        ui.colored_label(
                            match task.priority {
                                Priority::Critical => egui::Color32::RED,
                                Priority::High => egui::Color32::ORANGE,
                                Priority::Medium => egui::Color32::YELLOW,
                                Priority::Low => egui::Color32::GREEN,
                            },
                            format!("{:?}", task.priority)
                        );
                    });
                }
            }

            // Live graph
            ui.separator();
            ui.heading("Agent CPU Usage");

            Plot::new("cpu_plot")
                .view_aspect(2.0)
                .show(ui, |plot_ui| {
                    for agent in &self.agents {
                        let points: PlotPoints = agent.cpu_history
                            .iter()
                            .enumerate()
                            .map(|(i, v)| [i as f64, *v as f64])
                            .collect();

                        plot_ui.line(Line::new(points).name(&agent.id));
                    }
                });
        });
    }
}

// Launch widget
pub fn show_task_board(tasks: Vec<Task>) -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Colony Task Board",
        options,
        Box::new(|_cc| Box::new(TaskBoardWidget {
            tasks,
            selected: None,
        })),
    )?;

    Ok(())
}
```

**Features**:
- Native window (no browser)
- ~2-3MB memory overhead
- Cross-platform (Windows, Mac, Linux)
- Can also render to web (WebAssembly)
- Rich widgets: graphs, tables, forms, plots
- Immediate mode = simple code

**Example Usage**:
```rust
// In TUI, when user presses 'W'
Tab::WebUI => {
    let tasks = load_tasks()?;

    // Spawn in background thread so TUI continues
    thread::spawn(move || {
        show_task_board(tasks).ok();
    });
}
```

---

## Option 4: Ratatui + tui-widget-list (Enhanced TUI)

Enhance existing TUI with advanced widgets:

```rust
use ratatui::{
    widgets::{Block, Borders, Sparkline, BarChart, Table},
    layout::{Layout, Constraint, Direction},
};

pub fn render_advanced_dashboard(f: &mut Frame, data: &ColonyData) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(f.size());

    // Sparkline for CPU
    let sparkline = Sparkline::default()
        .block(Block::default().title("CPU Usage").borders(Borders::ALL))
        .data(&data.cpu_history)
        .style(Style::default().fg(Color::Green));

    f.render_widget(sparkline, chunks[0]);

    // Bar chart for tasks
    let bar_data = vec![
        ("Pending", data.pending_count),
        ("Active", data.active_count),
        ("Blocked", data.blocked_count),
        ("Done", data.completed_count),
    ];

    let barchart = BarChart::default()
        .block(Block::default().title("Tasks").borders(Borders::ALL))
        .data(&bar_data)
        .bar_width(9)
        .bar_gap(2)
        .bar_style(Style::default().fg(Color::Yellow))
        .value_style(Style::default().fg(Color::Black).bg(Color::Yellow));

    f.render_widget(barchart, chunks[1]);

    // Table for agents
    let rows = data.agents.iter().map(|agent| {
        Row::new(vec![
            agent.id.clone(),
            agent.status.to_string(),
            format!("{}%", agent.cpu_usage),
        ])
    });

    let table = Table::new(rows)
        .header(Row::new(vec!["Agent", "Status", "CPU"]))
        .block(Block::default().title("Agents").borders(Borders::ALL))
        .widths(&[
            Constraint::Percentage(40),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
        ]);

    f.render_widget(table, chunks[2]);
}
```

**Additions**:
- `ratatui-image` - Show images inline (using terminal graphics)
- `tui-textarea` - Rich text editing
- `tui-tree-widget` - Tree views for dependencies

---

## Option 5: Embedded Webview (Lighter than Browser)

Use system webview instead of full Chromium:

```rust
use wry::{
    application::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
    webview::WebViewBuilder,
};

pub fn show_task_graph() -> Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Colony Task Graph")
        .build(&event_loop)?;

    let html = include_str!("../../../examples/web_ui_poc/static/task-graph.html");

    let _webview = WebViewBuilder::new(window)?
        .with_html(html)?
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
```

**Size**: ~500KB (uses system webview)
**Pros**: Can reuse HTML/CSS/JS, lighter than browser
**Cons**: Still requires webview runtime

---

## Comparison Matrix

| Option | Memory | Startup | Complexity | Rich UI | Terminal | Cross-Platform |
|--------|--------|---------|------------|---------|----------|----------------|
| **Terminal Graphics** | ~0MB | <10ms | Low | Limited | ✅ | Partial |
| **Plotters ASCII** | ~0MB | <10ms | Low | Charts only | ✅ | ✅ |
| **Plotters + Image** | ~5MB | ~50ms | Medium | Charts | Via protocol | ✅ |
| **egui** | ~2-3MB | ~100ms | Medium | ✅ Full | ❌ Window | ✅ |
| **Enhanced Ratatui** | ~0MB | <10ms | Medium | Moderate | ✅ | ✅ |
| **Embedded Webview** | ~20MB | ~200ms | Medium | ✅ Full | ❌ Window | ✅ |
| **Full Browser** | ~100MB+ | ~1s | Low | ✅ Full | ❌ | ✅ |

---

## Recommended Approach: Hybrid Strategy

### Tier 1: Terminal-Native (Default)
**For most users, works everywhere**

```rust
// Enhanced ratatui with:
- Sparklines for metrics
- Bar charts for stats
- Tables with colors
- ASCII graphs via plotters
```

### Tier 2: Rich Terminal (Auto-detect)
**If terminal supports graphics protocols**

```rust
// Check terminal capabilities
if supports_kitty_graphics() || supports_sixel() {
    // Render beautiful charts using plotters + viuer
    render_graph_with_terminal_graphics()?;
} else {
    // Fall back to ASCII
    render_ascii_graph()?;
}
```

### Tier 3: Native Window (On Demand)
**When user needs interactive widgets**

```rust
// Press 'W' in TUI -> launch egui window
// Runs alongside TUI in separate thread
// User can interact with both simultaneously
```

---

## Implementation Example

```rust
// src/colony/ui/mod.rs
pub enum UIMode {
    TerminalAscii,      // Works everywhere
    TerminalGraphics,   // Kitty, Sixel, iTerm2
    NativeWindow,       // egui window
}

impl UIMode {
    pub fn detect() -> Self {
        if env::var("TERM").ok().as_deref() == Some("xterm-kitty") {
            UIMode::TerminalGraphics
        } else if supports_sixel() {
            UIMode::TerminalGraphics
        } else {
            UIMode::TerminalAscii
        }
    }
}

pub struct UIRenderer {
    mode: UIMode,
}

impl UIRenderer {
    pub fn render_task_graph(&self, tasks: &[Task]) -> Result<()> {
        match self.mode {
            UIMode::TerminalAscii => {
                self.render_ascii_graph(tasks)?;
            }
            UIMode::TerminalGraphics => {
                // Generate PNG with plotters
                let path = self.generate_chart_png(tasks)?;
                // Show in terminal
                viuer::print_from_file(&path, &Default::default())?;
            }
            UIMode::NativeWindow => {
                // Launch egui in background thread
                self.launch_native_window(tasks)?;
            }
        }
        Ok(())
    }
}
```

---

## Concrete Recommendations

### For Colony's Use Cases:

1. **Task Dependency Graph**
   - **Primary**: Plotters to PNG + terminal graphics protocol
   - **Fallback**: ASCII graph using `petgraph` + custom renderer
   - **On-demand**: egui interactive graph (press 'G')

2. **Live Metrics Dashboard**
   - **Primary**: Enhanced ratatui with sparklines/barcharts
   - **Already have this!** Just enhance existing TUI

3. **Complex Forms**
   - **Primary**: Enhanced ratatui multi-step dialogs (better UX)
   - **Alternative**: egui native window with real form widgets

4. **Agent Logs**
   - **Primary**: Ratatui with `tui-textarea` (search, filter)
   - **Already works!** Just add features

### Implementation Priority:

```
Week 1: Enhanced Ratatui
  - Add sparklines, bar charts
  - Better form widgets
  - Rich tables with sorting

Week 2: Terminal Graphics
  - Detect terminal capabilities
  - Plotters integration
  - Auto-fallback to ASCII

Week 3: egui Window (Optional)
  - Launch on 'W' key
  - Interactive graph editor
  - Form builder for complex inputs

Week 4: Polish
  - Auto-detection logic
  - User preferences
  - Documentation
```

---

## Code Size Comparison

| Approach | Binary Size Increase | Dependencies |
|----------|---------------------|--------------|
| Enhanced Ratatui | +~50KB | 0 new (already have ratatui) |
| Plotters | +~200KB | 1 (plotters) |
| Terminal Graphics | +~100KB | 1 (viuer) |
| egui | +~1.5MB | 2 (eframe, egui) |
| Webview | +~500KB | 1 (wry) |

---

## Conclusion

**Best Approach for Colony**:

1. **Start with Enhanced Ratatui** (Week 1)
   - Zero new dependencies
   - Works everywhere
   - Immediate improvement

2. **Add Terminal Graphics** (Week 2)
   - Small dependency
   - Beautiful charts for supported terminals
   - Graceful fallback

3. **egui as Optional Feature** (Week 3)
   - `cargo build --features gui`
   - For users who want rich interactive UIs
   - Doesn't bloat default build

This gives you:
- ✅ No browser required
- ✅ Lightweight (2-3MB total)
- ✅ Works in terminal
- ✅ Beautiful visualizations
- ✅ Optional native GUI
- ✅ Graceful degradation

**No manifest.build needed!** - Pure Rust solution that's lighter and more integrated.
