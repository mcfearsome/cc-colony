# CLI Reference Overview

Colony provides a comprehensive command-line interface for managing multi-agent systems.

## Command Structure

```bash
colony [COMMAND] [SUBCOMMAND] [OPTIONS]
```

## Available Commands

### Setup & Management
- `colony init` - Initialize a new colony
- `colony start` - Start all agents
- `colony stop` - Stop agents
- `colony destroy` - Destroy colony and cleanup

### Monitoring
- `colony status` - Show agent status
- `colony health` - System health check
- `colony tui` - Interactive TUI dashboard
- `colony logs` - View agent logs
- `colony attach` - Attach to tmux session

### Agent Templates
- `colony template list` - List templates
- `colony template show` - Show template details
- `colony template install` - Install built-in template
- `colony template builtin` - List built-in templates

### Plugins
- `colony plugin list` - List plugins
- `colony plugin show` - Show plugin details
- `colony plugin enable` - Enable a plugin
- `colony plugin disable` - Disable a plugin

### Workflows
- `colony workflow list` - List workflows
- `colony workflow show` - Show workflow details
- `colony workflow run` - Run a workflow
- `colony workflow status` - Check run status
- `colony workflow history` - View run history
- `colony workflow cancel` - Cancel a run

### State Management
- `colony state task` - Manage tasks
- `colony state workflow` - Manage workflow state
- `colony state memory` - Manage shared memory
- `colony state pull` - Pull from remote
- `colony state push` - Push to remote
- `colony state sync` - Full sync

### Communication
- `colony broadcast` - Broadcast message to all agents
- `colony messages list` - List agent messages
- `colony messages all` - List all messages

## Global Options

```bash
--help, -h     Show help information
--version, -V  Show version information
```

## Common Patterns

### Quick Status Check
```bash
colony status && colony health
```

### View Recent Errors
```bash
colony logs --level error --lines 50
```

### Start and Monitor
```bash
colony start && colony tui
```

### Sync and Stop
```bash
colony state sync && colony stop
```

## Exit Codes

- `0` - Success
- `1` - General error
- `2` - Command error
- `130` - Interrupted (Ctrl+C)

## Environment Variables

```bash
# Colony configuration file (default: colony.yml)
export COLONY_CONFIG=/path/to/config.yml

# Log level (default: info)
export COLONY_LOG_LEVEL=debug

# Tmux session name (default: colony)
export COLONY_TMUX_SESSION=my-session
```

## Shell Completion

Generate shell completions:

```bash
# Bash
colony completions bash > ~/.local/share/bash-completion/completions/colony

# Zsh
colony completions zsh > ~/.zsh/completions/_colony

# Fish
colony completions fish > ~/.config/fish/completions/colony.fish
```

## Next Steps

- [colony init](./init.md) - Initialize a colony
- [colony start](./start.md) - Start agents
- [colony template](./template.md) - Manage templates
- [colony workflow](./workflow.md) - Manage workflows
