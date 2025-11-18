# Quick Start

Get started with Colony in 5 minutes!

## Step 1: Initialize a Colony

Navigate to your project directory and initialize:

```bash
cd /path/to/your/project
colony init
```

This creates a `colony.yml` configuration file with default settings.

## Step 2: Configure Your Agents

Edit `colony.yml` to define your agents:

```yaml
agents:
  - id: main-dev
    worktree_branch: feature/main-work
    startup_prompt: |
      You are the main development agent. Focus on implementing
      core features and coordinating with other agents.

  - id: code-reviewer
    worktree_branch: review/automated
    template: code-reviewer
    startup_prompt: |
      Review code changes for quality and best practices.

  - id: test-writer
    worktree_branch: test/automated
    template: test-engineer
    startup_prompt: |
      Write comprehensive tests for new features.
```

## Step 3: Start the Colony

```bash
colony start
```

This command:
1. Creates a tmux session named `colony`
2. Sets up git worktrees for each agent
3. Launches Claude Code instances in separate panes
4. Initializes shared state

## Step 4: Monitor Your Agents

### Option 1: Attach to tmux
```bash
colony attach
```

View agents working in real-time. Press `Ctrl+b d` to detach.

### Option 2: Use the TUI
```bash
colony tui
```

Interactive dashboard with:
- Agent status and health
- Real-time metrics
- Task queue visualization
- Log viewing

### Option 3: Check Status
```bash
colony status
```

Quick overview of all agents.

## Step 5: View Logs

```bash
# View all logs
colony logs

# View specific agent
colony logs agent-id

# Filter by level
colony logs --level error

# Follow logs in real-time
colony logs --follow
```

## Step 6: Work with State

### Create Tasks
```bash
# Add a new task
colony state task add "Implement login feature" \
  --description "Add OAuth login support" \
  --blockers task-123

# List tasks
colony state task list

# List ready tasks (no blockers)
colony state task ready
```

### Sync State
```bash
# Pull latest state from remote
colony state pull

# Push local changes
colony state push

# Full sync (pull + push)
colony state sync
```

## Step 7: Stop When Done

```bash
# Stop all agents
colony stop

# Stop specific agent
colony stop agent-id
```

## Example: Code Review Workflow

Here's a common workflow for automated code review:

```bash
# 1. Initialize colony
colony init

# 2. Install code-reviewer template
colony template install code-reviewer

# 3. Configure agents
cat > colony.yml <<EOF
agents:
  - id: main-dev
    worktree_branch: feature/new-api
  - id: reviewer
    worktree_branch: review/api
    template: code-reviewer
EOF

# 4. Start colony
colony start

# 5. Create a task for the main dev
colony state task add "Implement new API endpoint"

# 6. Monitor progress
colony tui

# 7. When done, sync state and stop
colony state sync
colony stop
```

## Next Steps

- [Configuration](./configuration.md) - Learn about advanced configuration
- [Core Concepts](../concepts/agents.md) - Understand agents and colonies
- [Templates](../concepts/templates.md) - Use and create templates
- [Workflows](../concepts/workflows.md) - Define complex workflows
