# Webview Dashboard Guide

The Colony dashboard provides a rich web-based UI for monitoring and controlling your colony, running in a lightweight embedded webview instead of a full browser.

## Features

- 📊 **Real-time Statistics** - Task counts, agent status, completion rates
- ✅ **Task Management** - View all tasks with status indicators
- 🤖 **Agent Monitoring** - See agent status and current tasks
- 🎨 **Modern UI** - Professional dashboard with dark theme
- ⚡ **Lightweight** - ~20MB vs 100MB+ for full browser
- 🚀 **Fast** - ~200ms startup vs ~1s for browser

## Installation

### Prerequisites

The dashboard requires webkit2gtk on Linux:

```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-dev

# Fedora
sudo dnf install webkit2gtk4.1-devel

# Arch
sudo pacman -S webkit2gtk-4.1
```

**macOS** and **Windows**: No extra dependencies needed (uses system webview).

### Building with Webview Support

The dashboard is an optional feature. Build with:

```bash
# Build with webview support
cargo build --release --features webview

# Or install with webview
cargo install --path . --features webview
```

## Usage

### Launch the Dashboard

```bash
# Start the dashboard
colony dashboard
```

This opens a native window with the Colony dashboard.

### Features

#### Overview Tab
- Quick statistics (total tasks, agents, in-progress, completed)
- Recent tasks list
- Active agents list

#### Tasks Tab
- All tasks with color-coded status:
  - 🟡 **Pending** - Not started
  - 🟢 **In Progress** - Currently being worked on
  - 🔴 **Blocked** - Waiting on dependencies
  - ⚫ **Completed** - Done
- Priority badges (Critical, High, Medium, Low)
- Assigned agent information

#### Agents Tab
- All agents with status:
  - 🟢 **Running** - Active and working
  - ⚫ **Stopped** - Not running
  - 🔴 **Error** - Has errors
- Current task information
- Agent roles

### Auto-Refresh

The dashboard automatically refreshes data every 5 seconds to stay in sync with your colony.

### Manual Refresh

Click the "🔄 Refresh" button in the top-right to manually update data.

## Data Source

The dashboard loads data using a custom `colony://` protocol that reads directly from the colony state files:

- `colony://tasks` - Load all tasks
- `colony://agents` - Load all agents
- `colony://stats` - Load statistics

This provides real-time data without requiring a web server.

## Architecture

```
┌─────────────────────────────────────┐
│      Colony Dashboard (Webview)     │
│                                      │
│  ┌────────────────────────────────┐ │
│  │  HTML/CSS/JS Dashboard         │ │
│  │  (dashboard.html)              │ │
│  └─────────────┬──────────────────┘ │
│                │                     │
│                │ fetch('colony://') │
│                ↓                     │
│  ┌────────────────────────────────┐ │
│  │  Rust Backend                  │ │
│  │  (webview.rs)                  │ │
│  └─────────────┬──────────────────┘ │
│                │                     │
└────────────────┼─────────────────────┘
                 │
                 ↓
        ┌────────────────┐
        │  Colony State  │
        │  (.colony/)    │
        └────────────────┘
```

## Comparison to Browser Approach

| Feature | Embedded Webview | Full Browser |
|---------|-----------------|--------------|
| **Memory** | ~20MB | ~100MB+ |
| **Startup** | ~200ms | ~1s |
| **Installation** | System webview | Needs browser |
| **Integration** | Direct IPC | HTTP only |
| **Web Tech** | ✅ Full | ✅ Full |

## Troubleshooting

### "webkit2gtk not found" Error

Install the webkit2gtk development library:

```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-dev
```

### Dashboard Shows No Data

Make sure colony is initialized and has data:

```bash
# Initialize colony
colony init

# Start agents
colony start

# Create some tasks
colony tasks create test-task "Test Task" "Testing the dashboard"
```

### "Feature 'webview' not enabled" Error

Make sure you built with the webview feature:

```bash
cargo build --release --features webview
```

## Customization

The dashboard HTML is embedded in the binary at compile time. To customize:

1. Edit `src/colony/ui/dashboard.html`
2. Rebuild with `cargo build --features webview`

You can add:
- Additional views/tabs
- Custom charts (Chart.js, D3.js)
- Interactive controls
- WebSocket for live updates

## Future Enhancements

Planned features:
- [ ] Create tasks from dashboard
- [ ] Start/stop agents
- [ ] View agent logs
- [ ] Task dependency graph visualization
- [ ] Real-time WebSocket updates
- [ ] Agent performance charts

## See Also

- [Lightweight UI Options](./lightweight-ui-options.md) - Alternative UI approaches
- [UI Comparison](./ui-comparison-summary.md) - Complete comparison of all UI options
- [Final Recommendation](./FINAL-RECOMMENDATION.md) - Why we chose webview
