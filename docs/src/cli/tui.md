# colony tui

Interactive Terminal User Interface for monitoring and controlling your Colony.

## Overview

The TUI provides a real-time dashboard for managing your multi-agent system, with tabs for agents, tasks, messages, shared state, and help.

```bash
colony tui
```

## Features

- **Real-time monitoring** - Auto-refreshes every 2 seconds
- **Multi-tab interface** - Organized views for different aspects
- **Interactive dialogs** - Broadcast messages, create tasks, send messages
- **Keyboard-driven** - Fast navigation with vim-style keys
- **Status notifications** - Instant feedback on actions

## Tabs

### 1: Agents Tab

View all running agents and their current status.

**Information Displayed:**
- Agent ID and role
- Status (Running, Idle, Failed, Completed)
- Process ID (PID)
- Current task being worked on

**Status Indicators:**
- 🟢 **Running** - Agent is actively working
- ⚪ **Idle** - Agent is waiting for tasks
- 🔵 **Completed** - Agent finished its work
- 🔴 **Failed** - Agent encountered an error

### 2: Tasks Tab

Monitor the task queue and completion progress.

**Sections:**
- **Summary** - Visual progress bar and task counts
- **Task List** - Tasks grouped by status

**Task Statuses:**
- ⏳ **Pending** - Available to be claimed
- 👤 **Claimed** - Assigned to an agent
- 🔄 **In Progress** - Actively being worked on
- 🚫 **Blocked** - Waiting on dependencies
- ✅ **Completed** - Finished

**Metrics:**
- Total tasks
- Completion percentage
- Progress bar visualization

### 3: Messages Tab

View inter-agent and operator messages.

**Message Format:**
```
[TIME] SENDER → RECIPIENT: Message content
[TIME] SENDER [BROADCAST]: Message to all
```

**Features:**
- Chronological message history
- Broadcast message indicators
- Sender and recipient highlighting
- Recent messages display (most recent first)

### 4: State Tab

Monitor git-backed shared state (tasks and workflows).

**State Tasks:**
- Task ID and title
- Assigned agent
- Status indicators
- Blocker counts

**State Workflows:**
- Workflow ID and name
- Current step
- Running/completed status

**Note:** This tab only appears when shared state is enabled in `colony.yml`:

```yaml
shared_state:
  backend: git-backed
  location: in-repo
```

### 5: Help Tab

Complete keyboard shortcuts reference and feature documentation.

## Keyboard Shortcuts

### Navigation

| Key | Action |
|-----|--------|
| `1`, `2`, `3`, `4`, `5` | Switch to specific tab |
| `Tab` | Next tab |
| `Shift+Tab` | Previous tab |
| `↑` / `k` | Scroll up |
| `↓` / `j` | Scroll down |
| `PgUp` | Page up |
| `PgDn` | Page down |

### Actions

| Key | Action |
|-----|--------|
| `r` | Refresh data manually |
| `b` | Broadcast message to all agents |
| `t` | Create a new task |
| `m` | Send message to specific agent |
| `?` | Show help |

### General

| Key | Action |
|-----|--------|
| `q` | Quit TUI |
| `Ctrl+C` | Quit TUI |
| `Esc` | Cancel current dialog |

## Interactive Dialogs

### Broadcast Message (`b`)

Send a message to all agents simultaneously.

**Steps:**
1. Press `b` to open dialog
2. Type your message
3. Press `Enter` to send
4. Press `Esc` to cancel

**Example Use Cases:**
- Announce a priority change
- Share important findings
- Coordinate major changes
- Emergency notifications

### Create Task (`t`)

Add a new task to the queue with a multi-step form.

**Steps:**
1. Press `t` to open dialog
2. Enter each field:
   - **Task ID**: Unique identifier (e.g., `feature-123`)
   - **Title**: Short description (e.g., "Implement login API")
   - **Description**: Detailed explanation
   - **Assigned To**: Agent ID (optional, leave blank for "auto")
   - **Priority**: `low`, `medium`, `high`, or `critical` (default: medium)
3. Press `Enter` after each field
4. Task is created after final field
5. Press `Esc` to cancel at any step

**Features:**
- Progress indicator shows current step
- Previous inputs displayed during later steps
- Input validation
- Auto-refresh after creation

### Send Message (`m`)

Send a directed message to a specific agent.

**Steps:**
1. Press `m` to open dialog
2. Enter agent ID
3. Press `Enter`
4. Type your message
5. Press `Enter` to send
6. Press `Esc` to cancel

**Example Use Cases:**
- Request code review
- Ask for help with a blocker
- Share relevant findings
- Coordinate on shared files

## Metrics Panel

The metrics panel (always visible at top) shows:

**Agent Metrics:**
- Total agents
- Running count
- Idle count
- Failed count (if any)

**Task Metrics:**
- Total tasks
- Pending tasks
- In-progress tasks
- Completed tasks
- Total message count

**State Metrics** (when shared state enabled):
- State tasks (ready, in-progress, blocked, completed)
- State workflows (running, completed)

## Status Bar

The status bar (always visible at bottom) shows:

**Normal Mode:**
```
Status: 3 agents running | Shortcuts: q=Quit r=Refresh b=Broadcast t=Task m=Message ?=Help
```

**After Action:**
```
Status: Broadcast sent: Your message here
Error: Failed to create task: Task ID already exists
```

## Tips & Best Practices

### Monitoring Workflows

1. **Watch Agents tab** to see which agents are active
2. **Check Tasks tab** to monitor queue depth and completion
3. **Review Messages** to understand agent communication
4. **Monitor State** for cross-session coordination

### Creating Effective Tasks

- **Use descriptive Task IDs**: `feature-login` better than `task-1`
- **Write clear titles**: Short but informative
- **Provide context in description**: Help agents understand requirements
- **Set appropriate priority**: Reserve `critical` for urgent issues
- **Assign strategically**: Use `auto` for flexible claiming

### Broadcast Best Practices

- **Be concise**: Agents receive these as notifications
- **Be specific**: "PR #123 ready for review" vs "Something is ready"
- **Use for coordination**: Major changes, blockers, important updates
- **Don't spam**: Use targeted messages (`m`) for agent-specific communication

### Performance

The TUI is designed for efficiency:
- Auto-refreshes every 2 seconds
- Manual refresh with `r` for immediate updates
- Lightweight polling of filesystem
- No impact on agent performance

### Troubleshooting

**TUI won't start:**
```bash
# Check if colony is initialized
ls colony.yml

# Verify no permission issues
colony health
```

**No data showing:**
```bash
# Ensure colony is running
colony status

# Check for data in .colony/
ls -la .colony/
```

**Dialogs not working:**
- Ensure you're in normal mode (not another dialog)
- Press `Esc` to cancel any active dialog
- Check status bar for error messages

## Example Session

```bash
# Start TUI
colony tui

# In TUI:
# 1. Press '1' to view agents
# 2. Press 'b' to broadcast: "Starting feature work"
# 3. Press 't' to create task:
#    - ID: feature-api-auth
#    - Title: Implement API authentication
#    - Description: Add JWT-based auth to REST API
#    - Assigned: main-dev
#    - Priority: high
# 4. Press '2' to watch task progress
# 5. Press 'm' to message main-dev: "Check security guidelines"
# 6. Press 'r' to refresh manually
# 7. Press '3' to view message delivery
# 8. Press 'q' to quit
```

## Integration with Other Commands

The TUI complements command-line workflows:

```bash
# CLI: Check quick status
colony status

# TUI: Deep monitoring session
colony tui

# CLI: Add task via script
colony tasks create task-123 "Build API" "REST endpoints"

# TUI: View task in queue (Tab 2)
# TUI: Send message about task (Press 'm')

# CLI: View logs for debugging
colony logs --level error

# TUI: Monitor agent health (Tab 1)
```

## Advanced Usage

### Custom Refresh Interval

The TUI refreshes every 2 seconds by default. For near real-time monitoring on fast systems:

```bash
# Monitor in TUI
colony tui  # Default 2s refresh

# For critical monitoring, use status in a loop
watch -n 1 colony status
```

### Multi-Session Workflow

1. **Terminal 1**: Run `colony tui` for monitoring
2. **Terminal 2**: Use CLI commands for operations
3. **Terminal 3**: Run `colony attach` to watch agents

### Filtering and Focus

While the TUI shows all data by default, you can:
- Use `colony status` for quick checks
- Use `colony tasks list --status pending` for filtered views
- Use `colony logs agent-id` for specific agent details

Then return to TUI for overall coordination.

## See Also

- [colony status](./status.md) - Quick status command
- [colony logs](./logs.md) - View detailed logs
- [colony tasks](./overview.md#tasks) - Task management commands
- [Concepts: Agents](../concepts/agents.md) - Understanding agents
- [Examples](../examples/code-review.md) - Real-world workflows
