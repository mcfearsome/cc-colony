# Enhanced TUI Examples

Lightweight UI rendering alternatives that don't require a full browser.

## Examples

### 1. ASCII Task Graph (No Dependencies)
Pure Rust, works in any terminal:

```bash
cargo run --bin task_graph_ascii
```

**Output**:
- Topological sorted dependency graph
- Tree view of task relationships
- Color-coded status (Pending/In Progress/Blocked/Completed)
- Works on ANY terminal

### 2. Plotters Charts (Optional: `charts` feature)
Generate beautiful charts that can be displayed in terminal:

```bash
cargo run --bin plotters_demo --features charts
```

**Generates**:
- PNG charts (task_barchart.png, cpu_linechart.png)
- ASCII charts for universal compatibility
- Can be shown in terminal using:
  - `viuer` (Sixel support)
  - `kitty +kitten icat` (Kitty graphics protocol)
  - iTerm2 imgcat

### 3. egui Native GUI (Optional: `gui` feature)
Lightweight native window with rich widgets:

```bash
cargo run --bin egui_widget --features gui
```

**Features**:
- Native window (~2MB memory)
- Interactive task list with filtering
- Live CPU usage charts
- Task distribution bar chart
- Detail view with actions
- Auto-refresh every 1 second

## Feature Flags

```bash
# Default: ASCII only (zero dependencies)
cargo run --bin task_graph_ascii

# With charts support
cargo run --bin plotters_demo --features charts

# With GUI support
cargo run --bin egui_widget --features gui

# All features
cargo run --bin egui_widget --features all
```

## Comparison

| Example | Memory | Dependencies | Rich UI | Terminal | Interactive |
|---------|--------|--------------|---------|----------|-------------|
| ASCII Graph | ~0MB | 0 | Basic | ✅ | ❌ |
| Plotters | ~5MB | 1-2 | Charts | Via protocol | ❌ |
| egui | ~2-3MB | 3 | Full | ❌ Window | ✅ |

## Integration with Colony

### Option 1: Add to TUI (Recommended)
```rust
// In src/colony/tui/app.rs
match key.code {
    KeyCode::Char('g') | KeyCode::Char('G') => {
        // Show ASCII graph inline
        let renderer = AsciiGraphRenderer::new(80, 30);
        let graph = renderer.render_tree(&tasks);
        // Display in message area or new tab
    }
    // ...
}
```

### Option 2: Separate Command
```bash
colony tasks graph           # ASCII graph
colony tasks graph --gui     # Launch egui window
colony metrics chart         # Plotters chart
```

### Option 3: Auto-detect Terminal
```rust
fn show_task_graph(tasks: &[Task]) -> Result<()> {
    if terminal_supports_graphics() {
        // Generate PNG with plotters
        let path = generate_chart(tasks)?;
        // Show via viuer or kitty protocol
        show_in_terminal(&path)?;
    } else {
        // Fall back to ASCII
        let renderer = AsciiGraphRenderer::new(80, 30);
        println!("{}", renderer.render_tree(tasks));
    }
    Ok(())
}
```

## Terminal Graphics Support Detection

```rust
fn terminal_supports_graphics() -> bool {
    // Kitty
    if env::var("TERM").ok().as_deref() == Some("xterm-kitty") {
        return true;
    }

    // iTerm2
    if env::var("TERM_PROGRAM").ok().as_deref() == Some("iTerm.app") {
        return true;
    }

    // Sixel support check
    // (Query terminal capabilities)
    false
}
```

## Recommendations for Colony

### Phase 1: ASCII Graphics (Week 1)
- ✅ Zero dependencies
- ✅ Works everywhere
- ✅ Integrate into existing TUI
- Use: `task_graph_ascii.rs` as reference

### Phase 2: Terminal Graphics (Week 2)
- Add `plotters` + `viuer` dependencies
- Auto-detect terminal capabilities
- Fall back to ASCII if not supported
- Use: `plotters_demo.rs` as reference

### Phase 3: Optional GUI (Week 3)
- Make `egui` a feature flag
- Launch native window on demand (press 'W')
- For users who prefer graphical tools
- Use: `egui_widget.rs` as reference

## Size Impact

```bash
# Check binary sizes
cargo build --release
cargo build --release --features charts
cargo build --release --features gui

# Typical results:
# Base: 5MB
# With charts: 5.5MB (+500KB)
# With GUI: 7MB (+2MB)
```

## Live Demo

Run all examples:

```bash
# 1. ASCII (works on any terminal)
cargo run --bin task_graph_ascii

# 2. Charts (requires charts feature)
cargo run --bin plotters_demo --features charts
# Then view:
open task_barchart.png
# Or in terminal:
viuer task_barchart.png

# 3. Native GUI (requires gui feature)
cargo run --bin egui_widget --features gui
```

## Next Steps

1. **Test the examples** - Run each to see capabilities
2. **Choose approach** - ASCII + optional GUI recommended
3. **Integrate** - Add to colony TUI or as separate commands
4. **Document** - User guide for new visualizations

The egui example is particularly impressive - try it to see what's possible without a browser!
