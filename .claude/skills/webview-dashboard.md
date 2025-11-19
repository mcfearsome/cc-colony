---
skill_name: webview-dashboard
description: Guide for building and using the Colony webview dashboard
tags: [ui, webview, dashboard, visualization]
---

# Colony Webview Dashboard Skill

This skill teaches you how to build and use the Colony webview dashboard - a lightweight embedded web UI for rich visualizations.

## What is the Webview Dashboard?

The Colony webview dashboard is an **embedded web interface** that runs in a native window (not a browser):

- **Lightweight**: ~20MB memory (vs 100MB+ for browser)
- **Fast**: ~200ms startup (vs ~1s for browser)
- **Rich UI**: Full HTML/CSS/JS capabilities (Chart.js, D3.js, etc.)
- **Native Integration**: Direct access to colony data via custom `colony://` protocol
- **Optional**: Feature-flagged, doesn't bloat default builds

## When to Use

Use the webview dashboard when you need:

✅ **Rich Visualizations** - Charts, graphs, complex data tables
✅ **Interactive Forms** - Multi-field forms with validation
✅ **Real-time Updates** - Live agent status, task progress
✅ **Complex Interactions** - Drag-and-drop, filtering, sorting
✅ **Familiar Web Tech** - HTML/CSS/JS development

Don't use when:
❌ Working over SSH (use TUI instead)
❌ In headless/Docker environment (use CLI instead)
❌ Memory critical (use terminal UI instead)

## Building with Webview Support

### System Requirements

**Linux** (requires webkit2gtk):
```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-dev

# Fedora
sudo dnf install webkit2gtk4.1-devel

# Arch
sudo pacman -S webkit2gtk-4.1
```

**macOS**: No extra dependencies (uses built-in WebKit)

**Windows**: No extra dependencies (uses WebView2)

### Build Commands

```bash
# Build with webview feature
cargo build --release --features webview

# Install with webview feature
cargo install --path . --features webview

# Check if it works
./target/release/colony dashboard
```

## Launching the Dashboard

```bash
# Launch dashboard
colony dashboard
```

This opens a native window with:
- 📊 Overview tab (statistics, recent tasks, active agents)
- ✅ Tasks tab (all tasks with color-coded status)
- 🤖 Agents tab (all agents with current tasks)

## Architecture

```
User: colony dashboard
       ↓
┌──────────────────────────────────────┐
│   Native Window (wry crate)          │
│                                       │
│   ┌────────────────────────────────┐ │
│   │ dashboard.html (HTML/CSS/JS)   │ │
│   │                                │ │
│   │ fetch('colony://tasks')        │ ← Custom Protocol
│   │        ↓                       │ │
│   │ webview.rs (Rust)              │ │
│   │        ↓                       │ │
│   │ ColonyState::load()            │ │
│   │        ↓                       │ │
│   │ .colony/ directory             │ │
│   └────────────────────────────────┘ │
└──────────────────────────────────────┘
```

## Key Files

| File | Purpose | Lines |
|------|---------|-------|
| `src/colony/ui/webview.rs` | Rust backend, custom protocol handler | 243 |
| `src/colony/ui/dashboard.html` | Web UI (HTML/CSS/JS) | 463 |
| `src/colony/ui/mod.rs` | UI module entry point | 10 |
| `Cargo.toml` | Feature flag and dependencies | - |

## Custom Protocol: colony://

The dashboard uses a custom protocol to load data:

```javascript
// In JavaScript (dashboard.html)
const tasks = await fetch('colony://tasks').then(r => r.json());
const agents = await fetch('colony://agents').then(r => r.json());
const stats = await fetch('colony://stats').then(r => r.json());
```

**Available Endpoints:**
- `colony://tasks` - Returns all tasks as JSON
- `colony://agents` - Returns all agents as JSON
- `colony://stats` - Returns dashboard statistics as JSON

**How it works** (in `src/colony/ui/webview.rs`):
```rust
.with_custom_protocol("colony".into(), move |request| {
    match request.uri() {
        "colony://tasks" => {
            let state = state.lock().unwrap();
            let tasks = state.get_all_tasks();
            Ok(Response::json(tasks))
        }
        // ... other endpoints
    }
})
```

## Extending the Dashboard

### Adding a New View

1. **Edit dashboard.html** - Add new tab and render function:
```javascript
// Add to sidebar
<div class="nav-item" onclick="showView('metrics')">
    <span>📊</span>
    <span>Metrics</span>
</div>

// Add render function
function renderMetrics() {
    return `
        <div class="card">
            <h2>📊 Metrics</h2>
            <!-- Your content here -->
        </div>
    `;
}
```

2. **Rebuild**:
```bash
cargo build --features webview
```

### Adding a New Data Endpoint

1. **Edit webview.rs** - Add new endpoint to protocol handler:
```rust
"colony://metrics" => {
    let state = state.lock().unwrap();
    let metrics = get_metrics(&state);
    serde_json::to_string(&metrics).unwrap()
}
```

2. **Fetch in JavaScript**:
```javascript
const metrics = await fetch('colony://metrics').then(r => r.json());
```

### Adding Charts

The dashboard supports any JavaScript charting library:

**Chart.js** (recommended, easy to use):
```html
<!-- In dashboard.html -->
<script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.0/dist/chart.umd.min.js"></script>

<canvas id="myChart"></canvas>

<script>
const ctx = document.getElementById('myChart');
new Chart(ctx, {
    type: 'line',
    data: {
        labels: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri'],
        datasets: [{
            label: 'Tasks Completed',
            data: [12, 19, 3, 5, 2],
        }]
    }
});
</script>
```

**D3.js** (advanced, for complex visualizations):
```html
<script src="https://d3js.org/d3.v7.min.js"></script>
```

## IPC (JavaScript ↔ Rust Communication)

### Send Messages from JavaScript to Rust

```javascript
// In dashboard.html
window.ipc.postMessage('create_task:my-task-id');
window.ipc.postMessage('start_agent:backend-1');
```

### Handle in Rust

```rust
// In webview.rs
.with_ipc_handler(move |_window, message| {
    if let Some((cmd, args)) = message.split_once(':') {
        match cmd {
            "create_task" => {
                println!("Creating task: {}", args);
                // Call colony task creation logic
            }
            "start_agent" => {
                println!("Starting agent: {}", args);
                // Call colony agent start logic
            }
            _ => {}
        }
    }
})
```

## Auto-Refresh

The dashboard auto-refreshes every 5 seconds:

```javascript
// In dashboard.html
setInterval(refreshData, 5000); // 5 seconds

async function refreshData() {
    await loadData();
    showView(currentView);
}
```

## Debugging

### Enable Developer Tools

The webview supports dev tools for debugging:

```rust
// In webview.rs
WebViewBuilder::new(window)?
    .with_devtools(true) // Enable dev tools
    .build()?
```

Then **right-click in the window** and select "Inspect" to open dev tools.

### Console Logging

All `console.log()` calls in JavaScript appear in the terminal:

```javascript
// In dashboard.html
console.log('Loaded tasks:', tasks);
console.error('Error loading data:', error);
```

## Common Tasks

### Task: Display Task Dependency Graph

```javascript
// 1. Add D3.js to dashboard.html
<script src="https://d3js.org/d3.v7.min.js"></script>

// 2. Load tasks with dependencies
const tasks = await fetch('colony://tasks').then(r => r.json());

// 3. Create graph data
const nodes = tasks.map(t => ({ id: t.id, title: t.title }));
const links = tasks.flatMap(t =>
    (t.dependencies || []).map(d => ({ source: d, target: t.id }))
);

// 4. Render with D3.js force layout
// (see examples/web_ui_poc/static/task-graph.html for full example)
```

### Task: Add Live WebSocket Updates

```rust
// In webview.rs
use tokio_tungstenite::tungstenite::Message;

// Start WebSocket server
let ws_server = start_websocket_server(state.clone());

// Broadcast updates when state changes
ws_server.broadcast(Message::Text(
    serde_json::to_string(&updated_tasks)?
));
```

```javascript
// In dashboard.html
const ws = new WebSocket('ws://localhost:9001');
ws.onmessage = (event) => {
    const updated = JSON.parse(event.data);
    updateUI(updated);
};
```

## Performance Tips

1. **Lazy Load Data**: Only fetch what's needed for current view
2. **Virtual Scrolling**: For large task/agent lists (use libraries like `react-window`)
3. **Debounce Updates**: Don't refresh too frequently
4. **Cache Images**: Store generated charts to avoid regeneration

## Troubleshooting

### "webkit2gtk not found"
Install system dependencies (see "System Requirements" above).

### "No data shown"
Make sure colony is initialized and has data:
```bash
colony init
colony start
colony tasks create test "Test" "Testing"
```

### "Feature 'webview' not enabled"
Build with feature flag:
```bash
cargo build --features webview
```

### Dashboard crashes on Linux
Make sure you have the correct webkit2gtk version:
```bash
pkg-config --modversion webkit2gtk-4.1
# Should be >= 2.38
```

## Example: Complete Feature Addition

Let's add an agent performance chart:

**1. Add endpoint in webview.rs:**
```rust
"colony://agent-performance" => {
    let state = state.lock().unwrap();
    let agents = state.get_all_agents();
    let perf = agents.iter().map(|a| AgentPerformance {
        id: a.id.clone(),
        tasks_completed: a.completed_tasks,
        avg_duration: a.avg_task_duration,
    }).collect::<Vec<_>>();
    serde_json::to_string(&perf).unwrap()
}
```

**2. Add view in dashboard.html:**
```javascript
function renderPerformance() {
    return `
        <div class="card">
            <h2>📊 Agent Performance</h2>
            <canvas id="perfChart"></canvas>
        </div>
    `;
}

async function showPerformance() {
    const perf = await fetch('colony://agent-performance').then(r => r.json());

    new Chart(document.getElementById('perfChart'), {
        type: 'bar',
        data: {
            labels: perf.map(p => p.id),
            datasets: [{
                label: 'Tasks Completed',
                data: perf.map(p => p.tasks_completed),
                backgroundColor: '#4ec9b0'
            }]
        }
    });
}
```

**3. Rebuild and test:**
```bash
cargo build --features webview
colony dashboard
```

## Resources

- **Documentation**: `docs/webview-dashboard-guide.md`
- **Quick Start**: `WEBVIEW-QUICKSTART.md`
- **Examples**: `examples/webview_poc/`
- **Comparison**: `docs/ui-comparison-summary.md`

## Best Practices

✅ **DO:**
- Use the custom `colony://` protocol for data
- Add new views to existing dashboard.html
- Use Chart.js for simple charts, D3.js for complex
- Test in all target platforms (Linux, macOS, Windows)
- Keep dashboard.html under 1000 lines (split into files if needed)

❌ **DON'T:**
- Bundle large JavaScript libraries (use CDN)
- Store state in JavaScript (always fetch from Rust)
- Make synchronous blocking calls
- Forget to handle errors gracefully

## Summary

The webview dashboard provides:
- **Lightweight alternative** to full browser (80% lighter)
- **Rich UI capabilities** using standard web tech
- **Direct integration** with colony state via custom protocol
- **Optional feature** that doesn't bloat default builds

**To build**: `cargo build --features webview`
**To use**: `colony dashboard`
**To extend**: Edit `src/colony/ui/dashboard.html` and `webview.rs`

This enables agents working on UI improvements to quickly add rich visualizations and interactive features to Colony!
