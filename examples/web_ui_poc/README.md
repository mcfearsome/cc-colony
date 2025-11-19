# Web UI Proof of Concept

This directory contains a proof-of-concept for integrating manifest.build-style UIs into cc-colony.

## Quick Start

### Option 1: Using Manifest.build (Node.js)

```bash
# Install manifest.build
npm install -g @mnfst/manifest

# Start the UI server
cd examples/web_ui_poc
manifest dev

# Open browser to http://localhost:1111
# Default credentials: admin@manifest.build / admin
```

### Option 2: Using Axum (Rust - Coming Soon)

```bash
# Build the Rust web server
cargo build --example web_ui_server

# Start the server
cargo run --example web_ui_server

# Open browser to http://localhost:1111
```

## Files

- **manifest.yml** - Manifest.build configuration defining colony entities
- **web_ui_server.rs** - Rust-based alternative using Axum (to be created)
- **static/** - Static web assets (HTML, CSS, JS)
- **sync_colony_data.sh** - Script to sync colony data into manifest backend

## Integration with Colony

The web UI can be integrated into colony in several ways:

### 1. Subprocess Approach

```rust
// src/colony/web_ui/mod.rs
use std::process::{Command, Stdio};

pub fn start_web_ui() -> Result<Child, Error> {
    Command::new("manifest")
        .arg("dev")
        .current_dir("examples/web_ui_poc")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
}
```

### 2. TUI Integration

Add a new tab to the TUI that launches the browser:

```rust
// In src/colony/tui/app.rs
pub enum Tab {
    Agents,
    Tasks,
    Messages,
    State,
    Help,
    WebUI,  // NEW
}

// Handle WebUI tab selection
Tab::WebUI => {
    open::that("http://localhost:1111")?;
}
```

### 3. Data Synchronization

Sync colony data to manifest backend:

```bash
#!/bin/bash
# sync_colony_data.sh

COLONY_DIR=".colony"
MANIFEST_API="http://localhost:1111/api"

# Sync tasks
jq -c '.tasks[]' $COLONY_DIR/tasks.json | while read task; do
    curl -X POST $MANIFEST_API/tasks \
         -H "Content-Type: application/json" \
         -d "$task"
done

# Sync agents
jq -c '.agents[]' $COLONY_DIR/agents.json | while read agent; do
    curl -X POST $MANIFEST_API/agents \
         -H "Content-Type: application/json" \
         -d "$agent"
done
```

## Example Use Cases

### 1. Task Creation Form

Instead of 5-step CLI dialog, use manifest's auto-generated form:
- All fields visible at once
- Dropdown for agent selection (populated from active agents)
- Validation before submission
- Preview of task before creating

### 2. Dependency Graph Visualization

Add custom JavaScript to visualize task dependencies:

```html
<!-- static/task-graph.html -->
<!DOCTYPE html>
<html>
<head>
    <script src="https://d3js.org/d3.v7.min.js"></script>
</head>
<body>
    <div id="graph"></div>
    <script>
        fetch('/api/tasks')
            .then(r => r.json())
            .then(tasks => {
                // Render D3.js force-directed graph
                const nodes = tasks.map(t => ({id: t.id, title: t.title}));
                const links = tasks.flatMap(t =>
                    (t.dependencies || []).map(d => ({
                        source: t.id,
                        target: d
                    }))
                );

                // D3.js rendering code...
            });
    </script>
</body>
</html>
```

### 3. Live Agent Metrics

Real-time dashboard with WebSocket:

```javascript
// static/live-metrics.js
const ws = new WebSocket('ws://localhost:1111/ws');

ws.onmessage = (event) => {
    const metric = JSON.parse(event.data);
    updateChart(metric.agent_id, metric.metric_type, metric.value);
};
```

### 4. Agent-Triggered Prompts

Agent requests custom input:

```bash
# Agent code (via colony messages)
colony message send system "ui_prompt:deployment_config"

# Colony intercepts, launches form
# User fills form in browser
# Result sent back to agent as message
```

## Next Steps

1. **Test manifest.build** - Validate YAML schema works as expected
2. **Build Rust alternative** - Create `web_ui_server.rs` using Axum
3. **Implement sync script** - Make `sync_colony_data.sh` production-ready
4. **Add WebSocket support** - Enable real-time updates
5. **Create custom widgets** - Build task graph, metrics dashboard
6. **Integrate with TUI** - Add WebUI tab to main application

## Benefits Over CLI/TUI

| Feature | CLI/TUI | Web UI | Winner |
|---------|---------|--------|--------|
| Simple text input | ✅ Good | ✅ Good | Tie |
| Multi-field forms | ❌ 5 steps | ✅ 1 page | Web |
| Data validation | ⚠️ Basic | ✅ Rich | Web |
| Visualizations | ❌ ASCII | ✅ Charts | Web |
| Real-time updates | ⚠️ 2s poll | ✅ WebSocket | Web |
| Accessibility | ✅ Screen readers | ✅ ARIA | Tie |
| Remote access | ❌ SSH only | ✅ Browser | Web |
| Offline | ✅ Yes | ⚠️ Localhost | CLI |

## Security Considerations

- **Localhost only**: Bind to 127.0.0.1 by default
- **Auth tokens**: Reuse colony's OAuth system
- **CSRF protection**: Add tokens to forms
- **Rate limiting**: Prevent abuse
- **Input validation**: Sanitize all inputs

## Performance

- **Startup time**: +100-200ms for web server
- **Memory**: +20-50MB for Node.js/Rust server
- **Network**: Minimal (localhost only)
- **Response time**: <10ms for most API calls

## Fallback Strategy

If web UI fails:
1. Log error to colony logs
2. Fall back to CLI dialogs
3. Notify user via TUI
4. Continue colony operations normally

Web UI is **optional enhancement**, not core dependency.
