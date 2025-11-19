# Embedded Webview POC

This demonstrates using an **embedded webview** instead of a full browser - getting the best of both worlds:
- ✅ Full web tech stack (HTML/CSS/JS, Chart.js, D3.js, etc.)
- ✅ Much lighter than browser (~20MB vs ~100MB+)
- ✅ Faster startup (~200ms vs ~1s)
- ✅ Native window integration
- ✅ Bi-directional Rust ↔ JavaScript communication

## How It Works

Uses system webview instead of bundling Chromium:
- **macOS**: WebKit (same as Safari)
- **Linux**: WebKitGTK
- **Windows**: WebView2 (Microsoft Edge WebView2)

## Examples

### 1. Simple Webview
Basic example showing embedded HTML with nice UI:

```bash
cargo run --bin simple_webview
```

**Features**:
- Modern gradient UI
- Button interactions
- ~20MB memory usage
- No browser launch needed

### 2. Full Colony Dashboard
Complete dashboard with Rust ↔ JS communication:

```bash
cargo run --bin colony_dashboard_webview
```

**Features**:
- Loads data from Rust via custom `colony://` protocol
- Interactive task/agent lists
- Live charts (Chart.js)
- IPC for commands (JavaScript → Rust)
- Auto-refresh every 5 seconds

## Rust ↔ JavaScript Communication

### JavaScript → Rust (IPC)
```javascript
// In JavaScript
window.ipc.postMessage('create_task:my-task');
```

```rust
// In Rust
.with_ipc_handler(|_window, message| {
    if message.starts_with("create_task:") {
        let task_id = message.strip_prefix("create_task:").unwrap();
        create_task_in_colony(task_id);
    }
})
```

### Rust → JavaScript (Evaluate)
```rust
// Send data to JavaScript
webview.evaluate_script(&format!(
    "updateTasks({})",
    serde_json::to_string(&tasks)?
))?;
```

### Custom Protocol (colony://)
```rust
// In Rust
.with_custom_protocol("colony".into(), move |request| {
    match request.uri() {
        "colony://tasks" => {
            let json = load_colony_tasks();
            Ok(Response::builder()
                .header("Content-Type", "application/json")
                .body(json.into_bytes().into())
                .unwrap())
        }
        _ => Ok(Response::builder().status(404).body(Vec::new().into()).unwrap())
    }
})
```

```javascript
// In JavaScript
const tasks = await fetch('colony://tasks').then(r => r.json());
```

## Integration with Colony

### Option 1: Launch on Demand
```rust
// In src/colony/tui/app.rs
match key.code {
    KeyCode::Char('w') | KeyCode::Char('W') => {
        // Spawn webview in background thread
        std::thread::spawn(|| {
            show_webview_dashboard().ok();
        });
    }
}
```

### Option 2: Separate Command
```bash
colony dashboard        # Launch webview dashboard
colony tui              # Continue using TUI
```

### Option 3: Hybrid Mode
```rust
// TUI continues running
// Webview shows in separate window
// Both update same colony state
```

## Advantages Over Full Browser

| Feature | Webview | Full Browser |
|---------|---------|--------------|
| **Memory** | ~20MB | ~100MB+ |
| **Startup** | ~200ms | ~1s |
| **Bundled** | Uses system | Needs browser installed |
| **Size** | ~500KB dependency | N/A |
| **Integration** | Direct IPC | HTTP only |
| **Web Tech** | ✅ Full | ✅ Full |

## Advantages Over egui

| Feature | Webview | egui |
|---------|---------|------|
| **Web Skills** | Can use HTML/CSS/JS | Rust only |
| **Libraries** | Chart.js, D3.js, etc. | Limited |
| **Styling** | Full CSS | Manual |
| **Forms** | HTML forms | Manual widgets |
| **Reuse** | Can reuse web UI POC | New code |

## Comparison Summary

### Webview is BETTER than Browser when:
- ✅ You want lighter memory footprint
- ✅ You want faster startup
- ✅ You want better Rust integration
- ✅ You don't need multiple tabs/windows

### Webview is BETTER than egui when:
- ✅ You have web development skills
- ✅ You want to use existing web libraries (Chart.js, D3.js)
- ✅ You prefer declarative HTML/CSS
- ✅ You can reuse existing web assets

### Webview is WORSE than Terminal UI when:
- ❌ Need to work over SSH
- ❌ Need to work in Docker/containers
- ❌ User is in terminal-only environment
- ❌ Want zero window management

## Dependencies

```toml
[dependencies]
wry = "0.43"                    # ~500KB, webview wrapper
serde = "1.0"                   # JSON serialization
serde_json = "1.0"

[target.'cfg(target_os = "linux")'.dependencies]
webkit2gtk = "=2.0.1"           # Linux webview runtime
```

## Platform Requirements

### macOS
- WebKit (built-in)
- No extra requirements

### Linux
```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-dev

# Fedora
sudo dnf install webkit2gtk4.1-devel

# Arch
sudo pacman -S webkit2gtk-4.1
```

### Windows
- Requires Microsoft Edge WebView2 Runtime
- Usually pre-installed on Windows 11
- Auto-downloads if missing

## Build

```bash
# Simple example
cargo run --bin simple_webview

# Full dashboard
cargo run --bin colony_dashboard_webview

# Release build (much smaller)
cargo build --release --bin colony_dashboard_webview
```

## File Structure

```
examples/webview_poc/
├── Cargo.toml                    # Dependencies
├── simple_webview.rs             # Basic example
├── colony_dashboard.rs           # Full dashboard
├── dashboard.html                # HTML/CSS/JS for dashboard
└── README.md                     # This file
```

## Next Steps

1. **Test the examples** - Run both to see capabilities
2. **Compare with egui** - See which you prefer
3. **Integration** - Add to colony as feature flag:

```toml
[features]
webview = ["wry", "serde", "serde_json"]
gui = ["eframe", "egui"]  # Alternative

[dependencies]
wry = { version = "0.43", optional = true }
```

4. **Enhance** - Add more features:
   - WebSocket for live updates
   - File upload for configs
   - Drag-and-drop task management
   - Graph visualization with D3.js

## Recommendation

**Webview is excellent middle ground** between:
- Terminal UI (lightweight, works everywhere)
- egui (native, good Rust integration)
- Full browser (maximum flexibility)

Use webview if you want:
- ✅ Web development familiar to team
- ✅ Can reuse HTML/CSS/JS from web UI POC
- ✅ Need rich visualizations (Chart.js, D3.js)
- ✅ Want lighter than browser
- ✅ Don't mind ~20MB overhead vs terminal UI

**For colony, recommended architecture**:
1. **Terminal UI** (default) - Works everywhere
2. **Webview** (optional, `--features webview`) - Rich UI when needed
3. User chooses via: `colony tui` vs `colony dashboard`
