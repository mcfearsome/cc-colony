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

# Check for corrupted data
colony health
```

### Dialog not responding

**Problem**: Pressed `b`, `t`, or `m` but nothing happens

**Solution**:
- Ensure you're not already in a dialog (press `Esc` first)
- Check that colony is properly initialized (`ls colony.yml`)
- Verify agents are configured (`colony status`)
- Restart TUI: press `q`, then `colony tui`

### Can't type in dialog

**Problem**: Keyboard input not working in dialogs

**Solution**:
- Ensure dialog is actually open (yellow border should be visible)
- Try pressing `Esc` to cancel and re-open
- Check terminal emulator settings (some intercept keys)
- Verify terminal size is adequate (minimum 80x24)

### Dialog shows garbled text

**Problem**: Dialog content displays incorrectly

**Solution**:
```bash
# Set proper locale
export LC_ALL=en_US.UTF-8
export LANG=en_US.UTF-8

# Restart TUI
colony tui
```

### Status message not clearing

**Problem**: Status bar stuck showing old message

**Solution**:
- Messages clear automatically after actions
- Press `r` to refresh and clear status
- Navigate to different tab and back

### Tabs not switching

**Problem**: Number keys 1-5 don't switch tabs

**Solution**:
- Ensure TUI has focus (click the terminal window)
- Check if running inside another tmux session (can interfere with keys)
- Use `Tab` key to cycle through tabs instead
- Exit nested tmux if present: `tmux detach`

## Workflow Issues

### Workflow not starting

**Problem**: `colony workflow run` doesn't execute

**Solution**:
```bash
# Check workflow definition exists
ls .colony/workflows/

# Validate workflow syntax
colony workflow show workflow-name

# Check agents are available
colony status

# View workflow logs
colony logs --pattern "workflow"
```

### Workflow stuck on step

**Problem**: Workflow halts at specific step

**Solution**:
```bash
# Check workflow status
colony workflow status <run-id>

# View agent logs for that step
colony logs <agent-name>

# Check for blocked tasks
colony state task list --status blocked

# Cancel and retry
colony workflow cancel <run-id>
colony workflow run workflow-name
```

### Steps executing out of order

**Problem**: Dependencies not respected

**Solution**:
- Verify `depends_on` is correctly specified in workflow YAML
- Check for circular dependencies
- Ensure step names match exactly
- Review workflow validation: `colony workflow show workflow-name`

### Workflow timeout

**Problem**: "Workflow exceeded timeout"

**Solution**:
```yaml
# Increase timeout in workflow definition
steps:
  - name: long-running-step
    timeout: 30m  # Increase from default
    agent: worker
```

## Communication Issues

### Messages not delivered

**Problem**: Agent doesn't receive messages

**Solution**:
```bash
# Check message queue
colony messages all

# Verify agent is running
colony status

# Check message file permissions
ls -la .colony/messages/

# Manually inspect messages
cat .colony/messages/*.json
```

### Broadcast not visible to all agents

**Problem**: Some agents miss broadcasts

**Solution**:
- Broadcasts are stored as messages to "all"
- Agents must actively check messages
- Verify all agents are running: `colony status`
- Check message delivery in TUI: Tab 3

### Agent not responding to messages

**Problem**: Agent ignores directed messages

**Solution**:
- Agents must actively check their message queue
- Verify agent ID matches exactly (case-sensitive)
- Check agent's startup prompt includes message checking
- Review agent logs: `colony logs <agent-id>`

## Performance Issues

### Slow agent responses

**Problem**: Agents take long to respond

**Solution**:
```yaml
# Use faster models
agents:
  - id: quick-agent
    model: claude-sonnet-4  # Faster than opus
```

Additional steps:
- Reduce complexity of tasks
- Check network connection
- Monitor with `colony metrics show`
- Review API rate limits

### High memory usage

**Problem**: Colony uses too much memory

**Solution**:
```bash
# Check current usage
colony health

# Reduce number of concurrent agents
# Stop idle agents:
colony stop agent-id

# Use lighter models in colony.yml
```

### Disk space filling up

**Problem**: `.colony/` directory growing large

**Solution**:
```bash
# Check disk usage
du -sh .colony/*

# Clean old logs
find .colony/logs -type f -mtime +7 -delete

# Remove old metrics
colony metrics clear --older-than 7d

# Clean completed tasks
colony state task list --status completed |
  grep "$(date -d '30 days ago' +%Y-%m)" |
  xargs -I {} colony state task delete {}

# Prune git worktrees
git worktree prune
```

## Debugging Techniques

### Enable debug logging

```bash
# Set environment variable
export COLONY_LOG_LEVEL=debug

# Start colony
colony start

# Or for specific commands
COLONY_LOG_LEVEL=debug colony tui
```

### Inspect colony state

```bash
# View all state files
find .colony/ -type f | head -20

# Check configuration
cat colony.yml

# View tasks
cat .colony/state/tasks.jsonl

# Check messages
ls -la .colony/messages/

# Inspect metrics
ls -la .colony/metrics/
```

### Manual agent testing

```bash
# Attach to tmux and watch agent
colony attach

# Navigate to agent pane (Ctrl+b, arrow keys)
# Observe agent behavior in real-time

# Detach without stopping: Ctrl+b, d
```

### Network debugging

```bash
# Test Claude API connectivity
curl -I https://api.anthropic.com

# Check for proxy issues
echo $HTTP_PROXY
echo $HTTPS_PROXY

# Test with verbose output
claude --version --verbose
```

### File system debugging

```bash
# Check permissions
ls -la .colony/
ls -la .git/

# Verify disk space
df -h .

# Check inode usage (can cause issues)
df -i .

# Test file creation
touch .colony/test.txt
rm .colony/test.txt
```

### Database issues (SQLite)

If using SQLite backend for state:

```bash
# Check database integrity
sqlite3 .colony/state/colony.db "PRAGMA integrity_check;"

# View tables
sqlite3 .colony/state/colony.db ".tables"

# Query tasks
sqlite3 .colony/state/colony.db "SELECT * FROM tasks LIMIT 5;"

# Repair corrupted database (backup first!)
cp .colony/state/colony.db .colony/state/colony.db.backup
sqlite3 .colony/state/colony.db ".dump" | sqlite3 .colony/state/colony_new.db
mv .colony/state/colony_new.db .colony/state/colony.db
```

### Git state debugging

```bash
# Check git state
cd .colony/state
git status
git log --oneline -5

# Look for conflicts
git diff

# Check remote sync
git remote -v
git fetch origin
git log --oneline origin/main..HEAD

# Return to project root
cd ../..
```

### Resource monitoring

```bash
# Monitor CPU and memory
top -p $(pgrep -f colony)

# Monitor file handles
lsof | grep colony

# Monitor network connections
netstat -an | grep ESTABLISHED | grep claude

# Watch disk I/O
iotop -p $(pgrep -f colony)
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
