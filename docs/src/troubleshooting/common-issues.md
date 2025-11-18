# Common Issues

Solutions to frequently encountered problems.

## Installation Issues

### tmux not found

**Problem**: `colony start` fails with "tmux: command not found"

**Solution**:
```bash
# macOS
brew install tmux

# Ubuntu/Debian
sudo apt-get install tmux

# Fedora/RHEL
sudo dnf install tmux
```

### Rust/Cargo not installed

**Problem**: Cannot build colony from source

**Solution**:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

## Initialization Issues

### Not a Git repository

**Problem**: "colony must be run inside a Git repository"

**Solution**:
```bash
git init
git add .
git commit -m "Initial commit"
colony init
```

### colony.yml already exists

**Problem**: Wizard won't start because config exists

**Solution**:
```bash
# Backup existing config
mv colony.yml colony.yml.backup

# Run init
colony init

# Or use --force (if available)
rm colony.yml
colony init
```

## Start Issues

### Tmux session already exists

**Problem**: "tmux session 'colony-project' already exists"

**Solution**:
```bash
# Stop existing session
colony stop

# Or manually kill tmux session
tmux kill-session -t colony-project

# Then start again
colony start
```

### Worktree creation fails

**Problem**: "Cannot create worktree"

**Solution**:
```bash
# Ensure you have at least one commit
git status
git add .
git commit -m "Initial commit"

# Clean up old worktrees if any
git worktree prune

# Try again
colony start
```

### Claude Code not found

**Problem**: Claude Code doesn't launch in tmux panes

**Solution**:
```bash
# Ensure Claude Code CLI is installed
which claude

# Add to PATH if needed
export PATH="$PATH:/path/to/claude"

# Or configure in colony.yml (future feature)
```

## Runtime Issues

### Agent not responding

**Problem**: Agent pane shows no activity

**Solution**:
```bash
# Check logs
colony logs agent-id

# Check agent status
colony status

# Restart specific agent
colony stop agent-id
# Manually restart or use colony restart (if available)
```

### Git conflicts in worktrees

**Problem**: Agents have merge conflicts

**Solution**:
```bash
# Each agent works on their own branch
# Conflicts only occur when merging to main

# In agent's worktree:
git status
git merge-base HEAD main
git merge main

# Resolve conflicts and continue
```

### Permission denied errors

**Problem**: Cannot write to .colony directory

**Solution**:
```bash
# Check permissions
ls -la .colony

# Fix permissions
chmod -R u+w .colony

# Or remove and reinitialize
rm -rf .colony
colony init
```

## State Issues

### State sync fails

**Problem**: "Failed to sync state"

**Solution**:
```bash
# Check git status in .colony/state
cd .colony/state
git status

# Resolve any conflicts
git add .
git commit -m "Resolve state conflicts"

# Try sync again
cd ../..
colony state sync
```

### Task not showing up

**Problem**: Created task doesn't appear in queue

**Solution**:
```bash
# Force refresh
colony state pull

# Check task list
colony state task list

# Verify task was created
ls -la .colony/state/tasks.jsonl
```

## Metrics Issues

### No metrics available

**Problem**: `colony metrics list` shows "No metrics available"

**Solution**:
```bash
# Metrics are only available after agents run
colony start

# Or initialize sample metrics for testing
colony metrics init
```

### Metrics not persisting

**Problem**: Metrics disappear after restart

**Solution**: Metrics are currently in-memory only. Future versions will persist to disk.

## Template Issues

### Template not found

**Problem**: "Template 'xyz' not found"

**Solution**:
```bash
# List available templates
colony template builtin

# Install template first
colony template install code-reviewer

# Then use in config
```

### Template won't install

**Problem**: "Template already installed"

**Solution**:
```bash
# Remove existing template
rm -rf .colony/templates/template-name

# Install again
colony template install template-name
```

## TUI Issues

### TUI not updating

**Problem**: TUI shows stale data

**Solution**: Press 'r' to refresh, or restart TUI:
```bash
# Exit TUI (press 'q')
# Restart
colony tui
```

### TUI crashes

**Problem**: TUI exits unexpectedly

**Solution**:
```bash
# Check terminal compatibility
echo $TERM

# Try with different terminal
export TERM=xterm-256color
colony tui
```

## Performance Issues

### Slow agent responses

**Problem**: Agents take long to respond

**Solution**:
- Use faster model (Haiku instead of Opus)
- Reduce complexity of tasks
- Check network connection
- Monitor with `colony metrics`

### High memory usage

**Problem**: Colony uses too much memory

**Solution**:
```bash
# Check metrics
colony metrics show system.memory.used

# Reduce number of agents
# Or use lighter models (Haiku)
```

## Getting Help

If none of these solutions work:

1. Check logs: `colony logs --level error`
2. Check GitHub issues: [colony/issues](https://github.com/yourusername/cc-colony/issues)
3. Enable debug logging: `COLONY_LOG_LEVEL=debug colony start`
4. Create a minimal reproduction case

## See Also

- [FAQ](./faq.md) - Frequently asked questions
- [CLI Reference](../cli/overview.md) - Command details
- [Configuration](../getting-started/configuration.md) - Config options
