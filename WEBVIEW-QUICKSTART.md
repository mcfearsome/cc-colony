# Webview Dashboard - Quick Start

The Colony dashboard is now available! A rich web-based UI in a lightweight native window.

## Install System Dependencies (Linux Only)

```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-dev

# Fedora
sudo dnf install webkit2gtk4.1-devel

# Arch
sudo pacman -S webkit2gtk-4.1
```

**macOS/Windows**: No extra dependencies needed!

## Build with Webview Support

```bash
cargo build --release --features webview
```

## Launch the Dashboard

```bash
./target/release/colony dashboard
```

## What You Get

- 📊 Real-time statistics
- ✅ Color-coded task list
- 🤖 Agent status monitoring
- Auto-refresh every 5 seconds
- Modern dark theme UI

## Memory Footprint

- **Webview**: ~20MB
- **Full Browser**: ~100MB+
- **5x faster** startup than browser

## Example

```bash
# Initialize colony
colony init

# Start agents
colony start

# Create a task
colony tasks create my-task "Test Task" "Testing dashboard"

# Launch dashboard
colony dashboard
```

You'll see your tasks and agents in a beautiful web UI!

## Documentation

See [docs/webview-dashboard-guide.md](docs/webview-dashboard-guide.md) for complete documentation.
