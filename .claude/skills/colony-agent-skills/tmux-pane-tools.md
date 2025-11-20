---
name: tmux-pane-tools
description: Managing tmux panes for persistent tool sessions within Colony agents
---

# Tmux Pane Tools for Colony Agents

## Purpose

Enable agents to create and manage separate tmux panes for running persistent tools (editors, debuggers, REPLs) while keeping the main agent pane for conversation.

## Core Workflow

**1. Your Current Pane** - This is where you (Claude) are running. Never send commands here directly.

**2. Tool Panes** - Create separate panes for tools like nvim, debuggers, test runners, etc.

**3. Control via tmux send-keys** - Send keystrokes to tool panes to control them programmatically.

## Getting Your Pane Index

```bash
# Find your current pane index
tmux display-message -p '#{pane_index}'
```

Store this value - you'll need it to target other panes correctly.

## Creating Tool Panes

### Split Vertically (Side-by-side)
```bash
# Create a new pane to the right
tmux split-window -h -t $SESSION_NAME:0.$YOUR_PANE_INDEX
```

### Split Horizontally (Top-bottom)
```bash
# Create a new pane below
tmux split-window -v -t $SESSION_NAME:0.$YOUR_PANE_INDEX
```

### Get the New Pane Index
```bash
# After splitting, the new pane is the "last" pane
tmux display-message -p '#{pane_index}' -t $SESSION_NAME:0.\{last\}
```

## Sending Commands to Tool Panes

### Send Keys (Non-literal)
```bash
# Send command and execute it (C-m is Enter)
tmux send-keys -t $SESSION_NAME:0.$TOOL_PANE_INDEX "your command here" C-m
```

### Send Keys (Literal text)
```bash
# Send literal text without shell interpretation (-l flag)
tmux send-keys -t $SESSION_NAME:0.$TOOL_PANE_INDEX -l "text to type"
```

### Send Special Keys
- `C-m` - Enter
- `C-c` - Ctrl+C (interrupt)
- `C-d` - Ctrl+D (EOF)
- `Escape` - Escape key
- `BSpace` - Backspace
- `Space` - Space

### Send Multiple Keys in Sequence
```bash
# Example: Save and quit vim
tmux send-keys -t $SESSION_NAME:0.$PANE Escape
tmux send-keys -t $SESSION_NAME:0.$PANE ":wq" C-m
```

## Checking Pane Content

```bash
# Capture and view pane content
tmux capture-pane -t $SESSION_NAME:0.$PANE_INDEX -p

# Capture last N lines
tmux capture-pane -t $SESSION_NAME:0.$PANE_INDEX -p -S -20

# Check if pane still exists
tmux list-panes -t $SESSION_NAME -F '#{pane_index}' | grep "^$PANE_INDEX$"
```

## Managing Panes

### Switch Focus to Another Pane
```bash
tmux select-pane -t $SESSION_NAME:0.$PANE_INDEX
```

### Close a Tool Pane
```bash
# Gracefully close (send exit command first)
tmux send-keys -t $SESSION_NAME:0.$PANE_INDEX C-c
tmux send-keys -t $SESSION_NAME:0.$PANE_INDEX "exit" C-m

# Force kill if needed
tmux kill-pane -t $SESSION_NAME:0.$PANE_INDEX
```

### Resize Panes
```bash
# Make wider/narrower
tmux resize-pane -t $SESSION_NAME:0.$PANE_INDEX -L 10  # Shrink left
tmux resize-pane -t $SESSION_NAME:0.$PANE_INDEX -R 10  # Expand right

# Make taller/shorter
tmux resize-pane -t $SESSION_NAME:0.$PANE_INDEX -U 10  # Shrink up
tmux resize-pane -t $SESSION_NAME:0.$PANE_INDEX -D 10  # Expand down
```

## Environment Variables

Your environment has these tmux-related variables:
- `TMUX` - Set if running in tmux (contains socket path)
- `TMUX_PANE` - Your current pane ID (e.g., `%0`, `%1`)

## Session Name

For Colony, the session name is typically: `colony-<project-name>`

Example: `colony-gusto-web`

You can find it with:
```bash
tmux display-message -p '#{session_name}'
```

## Best Practices

**1. Always Check Pane Exists**
Before sending commands, verify the pane still exists.

**2. Don't Send to Your Own Pane**
Never send commands to your own pane index - it will interrupt your own input.

**3. Wait for Command Completion**
After sending a command, wait a moment, then capture the pane to see the result.

**4. Handle Errors Gracefully**
If a pane closes unexpectedly, detect it and recreate if needed.

**5. Clean Up on Completion**
When done with a tool, close its pane properly.

## Example: Create and Use a Tool Pane

```bash
# 1. Get your pane index
MY_PANE=$(tmux display-message -p '#{pane_index}')
SESSION=$(tmux display-message -p '#{session_name}')

# 2. Create tool pane (vertical split)
tmux split-window -v -t $SESSION:0.$MY_PANE

# 3. Get tool pane index
TOOL_PANE=$(tmux display-message -p '#{pane_index}' -t $SESSION:0.{last})

# 4. Send commands to tool pane
tmux send-keys -t $SESSION:0.$TOOL_PANE "echo 'Tool pane ready'" C-m

# 5. Check result
tmux capture-pane -t $SESSION:0.$TOOL_PANE -p | tail -5

# 6. When done, close it
tmux kill-pane -t $SESSION:0.$TOOL_PANE
```

## Integration with Other Skills

- Use with **nvim-pane-editing** skill to edit files in a separate pane
- Use with test runners, debuggers, or any interactive tool
- Keep conversation flow in your main pane
- Tool output doesn't clutter your conversation

## Limitations

- Cannot interact with tool panes that require mouse input
- Some TUI applications may not work well with send-keys
- Pane indices can change if panes are created/destroyed
- Always verify pane exists before sending commands
