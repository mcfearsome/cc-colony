---
name: nvim-pane-editing
description: Using nvim in a separate tmux pane for file editing within Colony agents
---

# Nvim Pane Editing for Colony Agents

## Purpose

Edit files in a persistent nvim session running in a separate tmux pane, controlled programmatically via send-keys from your main agent pane.

## Prerequisites

- Read the **tmux-pane-tools** skill first
- Understand pane indices and tmux send-keys

## Setup Workflow

### 1. Create Nvim Pane

```bash
# Get your current pane and session
MY_PANE=$(tmux display-message -p '#{pane_index}')
SESSION=$(tmux display-message -p '#{session_name}')

# Create pane for nvim (horizontal split below you)
tmux split-window -v -t $SESSION:0.$MY_PANE

# Get the nvim pane index
NVIM_PANE=$(tmux display-message -p '#{pane_index}' -t $SESSION:0.{last})

echo "Nvim pane created at index: $NVIM_PANE"
```

### 2. Start Nvim in the Pane

```bash
# Open a file
tmux send-keys -t $SESSION:0.$NVIM_PANE "nvim /path/to/file.ts" C-m

# Wait for nvim to load (1-2 seconds)
sleep 2

# Verify nvim is running
tmux capture-pane -t $SESSION:0.$NVIM_PANE -p | head -5
```

## Nvim Command Modes

### Normal Mode (Default)
- Commands start with `:`
- Navigation with `hjkl` or arrow keys
- Requires `Escape` to enter from other modes

### Insert Mode
- Press `i` to enter from Normal mode
- Type text directly
- Press `Escape` to exit back to Normal mode

### Command Mode
- Press `:` from Normal mode
- Enter commands like `:w` (save), `:q` (quit)
- Press `Enter` to execute
- Press `Escape` to cancel

## Common Nvim Operations

### Opening Files

```bash
# Open file in existing nvim session
tmux send-keys -t $SESSION:0.$NVIM_PANE Escape
tmux send-keys -t $SESSION:0.$NVIM_PANE ":e /path/to/file.ts" C-m
```

### Saving Files

```bash
# Save current file
tmux send-keys -t $SESSION:0.$NVIM_PANE Escape
tmux send-keys -t $SESSION:0.$NVIM_PANE ":w" C-m
```

### Reading File Content

```bash
# Capture visible content
tmux capture-pane -t $SESSION:0.$NVIM_PANE -p

# Capture with more history
tmux capture-pane -t $SESSION:0.$NVIM_PANE -p -S -100
```

### Navigating in Nvim

```bash
# Go to line 42
tmux send-keys -t $SESSION:0.$NVIM_PANE Escape
tmux send-keys -t $SESSION:0.$NVIM_PANE ":42" C-m

# Search for pattern
tmux send-keys -t $SESSION:0.$NVIM_PANE Escape
tmux send-keys -t $SESSION:0.$NVIM_PANE "/pattern" C-m

# Next search result
tmux send-keys -t $SESSION:0.$NVIM_PANE "n"

# Go to top of file
tmux send-keys -t $SESSION:0.$NVIM_PANE "gg"

# Go to bottom
tmux send-keys -t $SESSION:0.$NVIM_PANE "G"
```

### Editing Text

**Method 1: Line replacement**
```bash
# Delete current line and enter insert mode
tmux send-keys -t $SESSION:0.$NVIM_PANE Escape
tmux send-keys -t $SESSION:0.$NVIM_PANE "cc"
# Type new content (use -l for literal text)
tmux send-keys -t $SESSION:0.$NVIM_PANE -l "new line content"
tmux send-keys -t $SESSION:0.$NVIM_PANE Escape
```

**Method 2: Insert at cursor**
```bash
# Enter insert mode
tmux send-keys -t $SESSION:0.$NVIM_PANE Escape
tmux send-keys -t $SESSION:0.$NVIM_PANE "i"
# Type text
tmux send-keys -t $SESSION:0.$NVIM_PANE -l "text to insert"
tmux send-keys -t $SESSION:0.$NVIM_PANE Escape
```

**Method 3: Append to line**
```bash
# Append to end of line
tmux send-keys -t $SESSION:0.$NVIM_PANE Escape
tmux send-keys -t $SESSION:0.$NVIM_PANE "A"
tmux send-keys -t $SESSION:0.$NVIM_PANE -l "text to append"
tmux send-keys -t $SESSION:0.$NVIM_PANE Escape
```

### Multi-line Editing

```bash
# Insert multiple lines
tmux send-keys -t $SESSION:0.$NVIM_PANE Escape
tmux send-keys -t $SESSION:0.$NVIM_PANE "o"  # Open new line below
tmux send-keys -t $SESSION:0.$NVIM_PANE -l "line 1"
tmux send-keys -t $SESSION:0.$NVIM_PANE C-m  # New line in insert mode
tmux send-keys -t $SESSION:0.$NVIM_PANE -l "line 2"
tmux send-keys -t $SESSION:0.$NVIM_PANE Escape
```

### Deleting Text

```bash
# Delete current line
tmux send-keys -t $SESSION:0.$NVIM_PANE Escape
tmux send-keys -t $SESSION:0.$NVIM_PANE "dd"

# Delete 5 lines
tmux send-keys -t $SESSION:0.$NVIM_PANE Escape
tmux send-keys -t $SESSION:0.$NVIM_PANE "5dd"

# Delete to end of file
tmux send-keys -t $SESSION:0.$NVIM_PANE Escape
tmux send-keys -t $SESSION:0.$NVIM_PANE "dG"
```

### Copy/Paste

```bash
# Yank (copy) current line
tmux send-keys -t $SESSION:0.$NVIM_PANE Escape
tmux send-keys -t $SESSION:0.$NVIM_PANE "yy"

# Paste below
tmux send-keys -t $SESSION:0.$NVIM_PANE "p"

# Paste above
tmux send-keys -t $SESSION:0.$NVIM_PANE "P"
```

## Complete Example: Edit a File

```bash
#!/bin/bash
# Complete workflow: Open file, edit, save, close

SESSION=$(tmux display-message -p '#{session_name}')
MY_PANE=$(tmux display-message -p '#{pane_index}')

# Create nvim pane
tmux split-window -v -t $SESSION:0.$MY_PANE
NVIM_PANE=$(tmux display-message -p '#{pane_index}' -t $SESSION:0.{last})

# Open file
tmux send-keys -t $SESSION:0.$NVIM_PANE "nvim src/app.ts" C-m
sleep 2

# Go to line 10
tmux send-keys -t $SESSION:0.$NVIM_PANE Escape
tmux send-keys -t $SESSION:0.$NVIM_PANE ":10" C-m

# Replace line content
tmux send-keys -t $SESSION:0.$NVIM_PANE "cc"
tmux send-keys -t $SESSION:0.$NVIM_PANE -l "const newLine = 'updated';"
tmux send-keys -t $SESSION:0.$NVIM_PANE Escape

# Save
tmux send-keys -t $SESSION:0.$NVIM_PANE ":w" C-m
sleep 1

# Check for errors in status line
tmux capture-pane -t $SESSION:0.$NVIM_PANE -p | tail -1

# Quit nvim
tmux send-keys -t $SESSION:0.$NVIM_PANE ":q" C-m

# Close the pane
tmux kill-pane -t $SESSION:0.$NVIM_PANE
```

## Error Handling

### Check if Nvim is Still Running

```bash
# Check pane process
tmux list-panes -t $SESSION -F '#{pane_index} #{pane_current_command}' | grep "$NVIM_PANE nvim"
```

### Handle Unsaved Changes

```bash
# Force quit without saving
tmux send-keys -t $SESSION:0.$NVIM_PANE Escape
tmux send-keys -t $SESSION:0.$NVIM_PANE ":q!" C-m
```

### Recover from Errors

```bash
# If nvim is stuck, send Ctrl+C then try to quit
tmux send-keys -t $SESSION:0.$NVIM_PANE C-c
sleep 1
tmux send-keys -t $SESSION:0.$NVIM_PANE Escape
tmux send-keys -t $SESSION:0.$NVIM_PANE ":q!" C-m
```

## Advantages Over Direct File Tools

**✅ Persistent Session**: Editor stays open for multiple edits
**✅ Visual Feedback**: Can capture and see editor state
**✅ Interactive**: Can run nvim commands (:!, :make, etc.)
**✅ No Context Bloat**: File content doesn't fill your conversation
**✅ Real Editor**: Full nvim features (syntax highlighting, plugins, etc.)

## When to Use This vs Built-in Tools

**Use Nvim Pane When:**
- Making multiple edits to same file
- Need syntax highlighting/validation feedback
- Working with very large files
- Want to preview changes before committing

**Use Built-in Edit Tool When:**
- Single, simple edit
- File already loaded in context
- Need atomic operation
- Working with small files

## Tips

1. **Keep pane references** - Store `NVIM_PANE` in a variable for the session
2. **Always Escape first** - Ensures you're in Normal mode
3. **Verify after saves** - Capture status line to confirm `:w` succeeded
4. **Use relative paths** - They work from the agent's worktree directory
5. **Close when done** - Don't leave orphan panes running

## Common Patterns

### Quick Edit Pattern
```bash
# Open, edit line, save, quit
tmux send-keys -t $NVIM_PANE "nvim $FILE" C-m && sleep 2
tmux send-keys -t $NVIM_PANE Escape ":$LINE" C-m "cc" && sleep 1
tmux send-keys -t $NVIM_PANE -l "$NEW_CONTENT" && sleep 1
tmux send-keys -t $NVIM_PANE Escape ":wq" C-m
```

### Multi-file Session Pattern
```bash
# Open first file
tmux send-keys -t $NVIM_PANE "nvim file1.ts" C-m && sleep 2

# Edit it...

# Switch to another file
tmux send-keys -t $NVIM_PANE Escape ":e file2.ts" C-m && sleep 1

# Keep session alive for more edits
```

### Search and Replace Pattern
```bash
# Open file
tmux send-keys -t $NVIM_PANE "nvim $FILE" C-m && sleep 2

# Search and replace all
tmux send-keys -t $NVIM_PANE Escape
tmux send-keys -t $NVIM_PANE ":%s/oldtext/newtext/g" C-m

# Save
tmux send-keys -t $NVIM_PANE ":w" C-m
```
