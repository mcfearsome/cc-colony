# colony start

Start all agents in the colony.

## Synopsis

```bash
colony start [OPTIONS]
```

## Description

The `start` command launches all configured agents in a tmux session. Each agent runs in its own isolated tmux pane with its own git worktree.

## Options

### `--no-attach`

Start the colony without automatically attaching to the tmux session.

```bash
colony start --no-attach
```

## What Happens

When you run `colony start`:

1. **Tmux Session** - Creates a tmux session (default name: `colony-<project>`)
2. **Git Worktrees** - Sets up isolated git worktrees for each agent
3. **Agent Panes** - Creates a tmux pane for each agent
4. **Claude Code** - Launches Claude Code in each pane
5. **Startup Prompts** - Sends configured startup instructions to each agent
6. **Auto-attach** - Attaches to the session (unless `--no-attach`)

## Examples

### Start and Attach

```bash
colony start
```

This starts all agents and automatically attaches you to the tmux session.

### Start Without Attaching

```bash
colony start --no-attach
```

Agents start in the background. Attach later with:

```bash
colony attach
```

### Start and Monitor

```bash
colony start --no-attach && colony tui
```

Start agents in background and launch the TUI monitor.

## Tmux Layout

The default layout creates a pane for each agent:

```
┌─────────────┬─────────────┐
│   agent-1   │   agent-2   │
│             │             │
│             │             │
│             │             │
└─────────────┴─────────────┘
```

With more agents, tmux automatically tiles the panes.

## Tmux Key Bindings

Once attached to the session:

- `Ctrl+b d` - Detach from session (agents keep running)
- `Ctrl+b arrow` - Navigate between panes
- `Ctrl+b z` - Zoom/unzoom current pane
- `Ctrl+b [` - Enter scroll mode (q to exit)

## Agent Isolation

Each agent operates in isolation:

- **Separate worktree** - No git conflicts between agents
- **Independent directory** - Each agent has its own working directory
- **Isolated processes** - Each Claude Code instance runs independently
- **Unique configuration** - Each agent can have different MCP servers

## Checking Status

While agents are running:

```bash
# Quick status check
colony status

# Real-time TUI
colony tui

# View logs
colony logs agent-id
```

## Stopping Agents

```bash
# Stop all agents
colony stop

# Stop specific agent
colony stop agent-id
```

## Troubleshooting

### "tmux not found"

Install tmux:
```bash
# macOS
brew install tmux

# Ubuntu/Debian
sudo apt-get install tmux
```

### "tmux session already exists"

A colony session is already running. Options:

```bash
# Attach to existing session
colony attach

# Stop existing session first
colony stop
colony start
```

### Agent fails to start

Check logs:
```bash
colony logs agent-id
```

Common issues:
- Git worktree conflicts
- Invalid configuration
- MCP server errors

### "Cannot create worktree"

Ensure you're in a git repository with at least one commit:

```bash
git add .
git commit -m "Initial commit"
colony start
```

## Configuration

Control startup behavior in `colony.yml`:

```yaml
name: my-colony

agents:
  - id: agent-1
    role: Backend Engineer
    focus: API development
    model: claude-sonnet-4-20250514
    worktree: agent/backend
    startup_prompt: |
      Custom startup instructions here

# Optional: customize tmux behavior
# (future enhancement)
```

## Behind the Scenes

Colony performs these steps:

1. Load `colony.yml` configuration
2. Validate agent configurations
3. Create/verify git worktrees
4. Start tmux session
5. Create panes for each agent
6. Set up MCP server configurations
7. Launch Claude Code processes
8. Send startup prompts
9. Attach to session (if not `--no-attach`)

## See Also

- [colony stop](./stop.md) - Stop agents
- [colony attach](./attach.md) - Attach to session
- [colony status](./status.md) - Check agent status
- [colony tui](./tui.md) - Real-time monitoring
