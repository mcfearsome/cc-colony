# CC-Colony Manual Testing Guide

> A practical, hands-on guide for testing the colony CLI

This guide walks you through testing all cc-colony features manually. It's informal, practical, and designed to help you quickly verify everything works as expected.

## Prerequisites

Before you start testing:

- [ ] Install Rust and build the project: `cargo build --release`
- [ ] Ensure tmux is installed: `tmux -V` (should show version 2.0+)
- [ ] Ensure git is installed: `git --version`
- [ ] Create a test git repository (don't use your main project!)

**Quick setup:**
```bash
mkdir ~/colony-test
cd ~/colony-test
git init
git commit --allow-empty -m "Initial commit"
```

---

## Build and Install

```bash
# From the cc-colony repo root
cargo build --release

# Option 1: Run directly
./target/release/colony --help

# Option 2: Install to PATH
cargo install --path .
colony --help
```

**Expected output:**
```
Multi-agent orchestration for Claude Code on tmux

Usage: colony <COMMAND>

Commands:
  init       Initialize a new colony configuration
  start      Start all agents in the colony (requires tmux)
  attach     Attach to the tmux session to watch agents work
  tui        Interactive TUI for monitoring and controlling the colony
  status     Show status of running agents
  broadcast  Broadcast a message to all agents
  stop       Stop one or all agents
  logs       View agent logs
  destroy    Destroy the colony and clean up resources
  messages   View and manage messages
  tasks      List and manage tasks
  help       Print this message or the help of the given subcommand(s)
```

---

## Test Scenarios

### 1. Basic Initialization

**Test: Initialize a new colony**

```bash
cd ~/colony-test
colony init
```

**Expected results:**
- ✅ Creates `colony.yml` in current directory
- ✅ Shows success message: "Created colony.yml"
- ✅ Creates `.colony/tasks/` directories
- ✅ Displays next steps and example config

**Verify:**
```bash
ls -la colony.yml        # Should exist
cat colony.yml           # Should show 2 example agents
ls -la .colony/tasks/    # Should have pending/, claimed/, etc.
```

**Example colony.yml:**
```yaml
agents:
  - id: backend-1
    role: Backend Engineer
    focus: API endpoints and server logic
    model: claude-opus-4-20250514
  - id: frontend-1
    role: Frontend Engineer
    focus: React components and UI implementation
    model: claude-sonnet-4-20250514
```

---

**Test: Re-running init on existing colony**

```bash
colony init  # Run again
```

**Expected results:**
- ✅ Prompts: "colony.yml already exists. Overwrite? [y/N]"
- If you press `n`: Shows "Initialization cancelled"
- If you press `y`: Overwrites with new default config

---

**Test: Running init outside a git repo**

```bash
cd /tmp
mkdir not-a-git-repo
cd not-a-git-repo
colony init
```

**Expected results:**
- ❌ Error: "colony must be run inside a Git repository"

---

### 2. Starting Agents

**Test: Start agents (basic)**

```bash
cd ~/colony-test
colony start --no-attach  # Don't auto-attach for easier testing
```

**Expected results:**
- ✅ Checks tmux availability
- ✅ Creates tmux session named "colony-test" (or your colony name)
- ✅ Creates git worktrees in `.colony/worktrees/backend-1` and `.colony/worktrees/frontend-1`
- ✅ Spawns tmux panes for each agent
- ✅ Shows progress indicators

**Verify:**
```bash
tmux ls                                    # Should show colony session
ls -la .colony/worktrees/                 # Should have backend-1/ and frontend-1/
git worktree list                         # Should show all worktrees
```

---

**Test: Start without --no-attach**

```bash
colony destroy  # Clean up first (we'll test this later)
colony start    # Will auto-attach
```

**Expected results:**
- ✅ Starts agents AND automatically attaches to tmux session
- You should see tmux interface with multiple panes
- Press `Ctrl+b d` to detach from tmux

---

**Test: Start when already running**

```bash
colony start
```

**Expected results:**
- Should handle gracefully (either skip or warn that session exists)

---

**Test: Start with missing tmux**

```bash
# Temporarily rename tmux (requires sudo)
sudo mv /usr/bin/tmux /usr/bin/tmux.bak
colony start
```

**Expected results:**
- ❌ Error about tmux not being installed
- ✅ Offers to install tmux (on supported systems)

**Cleanup:**
```bash
sudo mv /usr/bin/tmux.bak /usr/bin/tmux
```

---

### 3. Checking Status

**Test: View agent status**

```bash
colony status
```

**Expected results:**
- ✅ Shows table or list of agents
- ✅ Displays agent ID, role, status (Running/Idle/etc.)
- ✅ Color-coded output

**Example output:**
```
Colony Status
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ID          ROLE               STATUS
backend-1   Backend Engineer   Running
frontend-1  Frontend Engineer  Running
```

---

**Test: Status when not running**

```bash
colony stop       # Stop all agents first
colony status
```

**Expected results:**
- ✅ Shows agents but with "Idle" or "Stopped" status
- OR: Shows "No agents running"

---

### 4. TUI (Interactive Dashboard)

**Test: Launch TUI**

```bash
colony start --no-attach  # Make sure agents are running
colony tui
```

**Expected results:**
- ✅ Launches full-screen terminal UI (using ratatui)
- ✅ Shows agent status, tasks, messages
- ✅ Updates in real-time
- ✅ Press `q` to quit

**What to check:**
- Agents are listed
- Status updates live
- UI is responsive
- No visual glitches

---

**Test: TUI with no colony.yml**

```bash
cd /tmp
colony tui
```

**Expected results:**
- ❌ Error: "colony.yml not found" or similar

---

### 5. Attaching to Tmux

**Test: Attach to running session**

```bash
colony attach
```

**Expected results:**
- ✅ Attaches to tmux session
- ✅ You can see all agent panes
- ✅ Can navigate with tmux commands:
  - `Ctrl+b arrow-keys`: Move between panes
  - `Ctrl+b d`: Detach
  - `Ctrl+b x`: Kill pane (careful!)

---

**Test: Attach when not running**

```bash
colony stop
colony attach
```

**Expected results:**
- ❌ Error: "No colony session found" or similar

---

### 6. Broadcasting Messages

**Test: Send broadcast message**

```bash
colony broadcast "Hello from the orchestrator!"
```

**Expected results:**
- ✅ Message saved to `.colony/messages/`
- ✅ All agents can receive the message
- ✅ Success confirmation shown

**Verify:**
```bash
ls -la .colony/messages/   # Should have message files
colony messages all        # Should show your broadcast
```

---

**Test: Broadcast with special characters**

```bash
colony broadcast "Test: quotes 'single' \"double\" & symbols $!"
```

**Expected results:**
- ✅ Message saved correctly without breaking shell
- ✅ Special characters escaped properly

---

### 7. Viewing Messages

**Test: List all messages**

```bash
colony messages all
```

**Expected results:**
- ✅ Shows all messages in the system
- ✅ Displays timestamp, sender, recipient, content
- ✅ Formatted nicely

---

**Test: List messages for specific agent**

```bash
colony messages list backend-1
```

**Expected results:**
- ✅ Shows only messages for backend-1
- ✅ Includes broadcasts (sent to "all")

---

**Test: Messages when empty**

```bash
# In a fresh colony
colony messages all
```

**Expected results:**
- ✅ Shows "No messages" or empty list
- ✅ Doesn't crash

---

### 8. Task Management

#### Creating Tasks

**Test: Create a basic task**

```bash
colony tasks create task-1 "Setup API" "Create REST API endpoints for user management"
```

**Expected results:**
- ✅ Task created successfully
- ✅ Saved to `.colony/tasks/pending/task-1.json`
- ✅ Success message shown

**Verify:**
```bash
ls .colony/tasks/pending/       # Should have task-1.json
colony tasks show task-1        # Should show task details
```

---

**Test: Create task with priority and assignment**

```bash
colony tasks create task-2 "Fix Bug" "Fix login validation bug" \
  --assigned-to backend-1 \
  --priority high
```

**Expected results:**
- ✅ Task created with priority and assignment
- ✅ Metadata saved correctly

---

**Test: Create task with duplicate ID**

```bash
colony tasks create task-1 "Duplicate" "This should fail"
```

**Expected results:**
- ❌ Error: "Task task-1 already exists" or similar

---

#### Listing Tasks

**Test: List all tasks**

```bash
colony tasks list
```

**Expected results:**
- ✅ Shows all tasks across all statuses
- ✅ Displays ID, title, status, assigned agent
- ✅ Color-coded by status

---

**Test: List tasks by status**

```bash
colony tasks list --status pending
colony tasks list --status claimed
colony tasks list --status completed
```

**Expected results:**
- ✅ Filters correctly by status
- ✅ Empty list if no tasks in that status

---

**Test: Compact view**

```bash
colony tasks list --compact
```

**Expected results:**
- ✅ Simplified output (less detail)
- ✅ Good for quick overview

---

#### Claiming Tasks

**Test: Claim a task**

```bash
colony tasks claim task-1 backend-1
```

**Expected results:**
- ✅ Task moved from `pending/` to `claimed/`
- ✅ Status updated to "claimed"
- ✅ Agent assignment recorded

**Verify:**
```bash
ls .colony/tasks/claimed/       # Should have task-1.json
colony tasks show task-1        # Status should be "claimed"
colony tasks agent backend-1    # Should list task-1
```

---

**Test: Claim already claimed task**

```bash
colony tasks claim task-1 frontend-1  # Try to claim with different agent
```

**Expected results:**
- ❌ Error: "Task already claimed" or allows re-claim (depending on implementation)

---

#### Updating Progress

**Test: Update task progress**

```bash
colony tasks progress task-1 50
```

**Expected results:**
- ✅ Progress updated to 50%
- ✅ Task metadata shows progress

**Verify:**
```bash
colony tasks show task-1  # Should show "Progress: 50%"
```

---

**Test: Invalid progress values**

```bash
colony tasks progress task-1 150   # Over 100
colony tasks progress task-1 -10   # Negative
```

**Expected results:**
- ❌ Error: "Progress must be 0-100" or type validation error

---

#### Blocking Tasks

**Test: Block a task**

```bash
colony tasks block task-1 "Waiting for API design approval"
```

**Expected results:**
- ✅ Task moved to `blocked/` directory
- ✅ Blocker reason saved
- ✅ Status updated to "blocked"

**Verify:**
```bash
ls .colony/tasks/blocked/       # Should have task-1.json
colony tasks show task-1        # Should show blocker reason
```

---

**Test: Unblock a task**

```bash
colony tasks unblock task-1
```

**Expected results:**
- ✅ Task moved back to `claimed/` or `in_progress/`
- ✅ Status updated
- ✅ Blocker reason cleared

---

#### Completing Tasks

**Test: Complete a task**

```bash
colony tasks complete task-1
```

**Expected results:**
- ✅ Task moved to `completed/` directory
- ✅ Status updated to "completed"
- ✅ Completion timestamp recorded

**Verify:**
```bash
ls .colony/tasks/completed/     # Should have task-1.json
colony tasks list --status completed  # Should show task-1
```

---

**Test: Complete already completed task**

```bash
colony tasks complete task-1
```

**Expected results:**
- ✅ Idempotent (no error, just confirmation) OR
- ⚠️  Warning: "Task already completed"

---

#### Canceling and Deleting Tasks

**Test: Cancel a task**

```bash
colony tasks create task-3 "Test Cancel" "This will be cancelled"
colony tasks cancel task-3
```

**Expected results:**
- ✅ Task status changed to "cancelled"
- ✅ Still exists in filesystem (not deleted)

---

**Test: Delete a task**

```bash
colony tasks delete task-3
```

**Expected results:**
- ✅ Task file removed from filesystem
- ✅ No longer appears in listings
- ⚠️  May prompt for confirmation

**Verify:**
```bash
colony tasks list         # Should not show task-3
colony tasks show task-3  # Should error: "Task not found"
```

---

#### Agent-Specific Task Views

**Test: List tasks for an agent**

```bash
colony tasks agent backend-1
```

**Expected results:**
- ✅ Shows all tasks assigned to backend-1
- ✅ Includes claimed, in-progress, and completed tasks

---

**Test: List claimable tasks**

```bash
colony tasks claimable backend-1
```

**Expected results:**
- ✅ Shows tasks that backend-1 can claim
- ✅ Only pending/unassigned tasks OR tasks assigned to backend-1

---

### 9. Viewing Logs

**Test: List all agent logs**

```bash
colony logs
```

**Expected results:**
- ✅ Lists available log files
- ✅ Shows agent IDs with log paths

---

**Test: View specific agent log**

```bash
colony logs backend-1
```

**Expected results:**
- ✅ Displays log file contents
- ✅ Shows Claude Code output/errors
- ✅ May tail the file (follow mode)

**Note:** Logs may be empty if agent hasn't started or logged anything yet.

---

**Test: View logs for non-existent agent**

```bash
colony logs nonexistent-agent
```

**Expected results:**
- ❌ Error: "Agent not found" or "No logs for agent"

---

### 10. Stopping Agents

**Test: Stop specific agent**

```bash
colony stop backend-1
```

**Expected results:**
- ✅ Stops only backend-1
- ✅ Kills the tmux pane
- ✅ Updates agent status

**Verify:**
```bash
colony status  # backend-1 should show as stopped
tmux ls        # Session still exists (other agents running)
```

---

**Test: Stop all agents**

```bash
colony stop
```

**Expected results:**
- ✅ Stops all running agents
- ✅ Kills all tmux panes
- ✅ Tmux session may still exist but be empty

**Verify:**
```bash
colony status   # All agents should be stopped
tmux ls         # May show empty session or no session
```

---

**Test: Stop when not running**

```bash
colony stop backend-1  # Already stopped
```

**Expected results:**
- ⚠️  Warning: "Agent not running" OR
- ✅ Idempotent (no error)

---

### 11. Destroying Colony

**Test: Destroy colony**

```bash
colony destroy
```

**Expected results:**
- ⚠️  Prompts for confirmation: "Are you sure? This will remove all worktrees and state. [y/N]"
- If you confirm:
  - ✅ Removes all git worktrees
  - ✅ Deletes `.colony/` directory
  - ✅ Kills tmux session
  - ✅ Shows cleanup progress

**Verify:**
```bash
ls -la .colony/         # Should not exist
git worktree list       # Should only show main worktree
tmux ls                 # Colony session should be gone
```

---

**Test: Destroy and cancel**

```bash
colony init             # Re-initialize
colony start --no-attach
colony destroy          # But press 'n' when prompted
```

**Expected results:**
- ⚠️  Shows confirmation prompt
- ✅ Cancels operation when you press 'n'
- ✅ Everything still intact

---

**Test: Destroy when not initialized**

```bash
cd /tmp/empty-dir
colony destroy
```

**Expected results:**
- ❌ Error: "No colony found" or similar

---

## Edge Cases and Error Scenarios

### Invalid Colony Config

**Test: Malformed YAML**

```bash
# Edit colony.yml and break the YAML syntax
echo "invalid: yaml: : :" > colony.yml
colony start
```

**Expected results:**
- ❌ YAML parse error with helpful message

---

**Test: Duplicate agent IDs**

```yaml
# Edit colony.yml
agents:
  - id: agent-1
    role: Developer
    focus: Backend
  - id: agent-1  # Duplicate!
    role: Tester
    focus: QA
```

```bash
colony start
```

**Expected results:**
- ❌ Error: "Duplicate agent ID: agent-1"

---

**Test: Invalid agent ID characters**

```yaml
agents:
  - id: "agent/../../../etc/passwd"  # Path traversal attempt
    role: Hacker
    focus: Evil
```

**Expected results:**
- ❌ Error: "Invalid agent ID" (should only allow alphanumeric, hyphens, underscores)

---

### Resource Cleanup

**Test: Orphaned worktrees**

```bash
# Manually delete .colony/ but keep worktrees
colony start
rm -rf .colony/
colony start
```

**Expected results:**
- Should handle gracefully (either reuse or warn about existing worktrees)

---

**Test: Disk space issues**

This is hard to test manually, but you could:
1. Fill up disk space (in a VM!)
2. Try creating tasks or starting agents
3. Should fail with clear error about disk space

---

### Concurrent Operations

**Test: Multiple terminal sessions**

```bash
# Terminal 1
colony status

# Terminal 2 (at the same time)
colony tasks create task-new "Test" "Concurrent access"
```

**Expected results:**
- ✅ Both operations succeed (if state management handles concurrency)
- OR: May have race conditions (document this!)

---

## Performance and Stress Testing

### Many Agents

**Test: 10+ agents**

Edit `colony.yml` to add 10 agents:
```yaml
agents:
  - id: agent-1
    role: Developer
    focus: Component 1
  - id: agent-2
    role: Developer
    focus: Component 2
  # ... add 8 more
```

```bash
colony start --no-attach
colony status
```

**What to check:**
- ✅ All agents start successfully
- ✅ Tmux handles multiple panes
- ✅ Worktrees created for all
- ⚠️  Check system resource usage (CPU, memory)
- ⚠️  May hit tmux pane limits (usually ~20-30)

---

### Many Tasks

**Test: 100+ tasks**

```bash
# Create 100 tasks
for i in {1..100}; do
  colony tasks create "task-$i" "Task $i" "Description for task $i"
done

# List them
colony tasks list
```

**What to check:**
- ✅ All tasks created
- ✅ Listing is reasonably fast (<1 second)
- ✅ No crashes or memory issues

---

### Large Messages

**Test: Long broadcast message**

```bash
# Create a very long message
LONG_MSG=$(python3 -c "print('A' * 10000)")
colony broadcast "$LONG_MSG"
```

**What to check:**
- ✅ Message saved successfully
- ✅ Can retrieve without truncation
- ✅ No buffer overflow issues

---

## Integration Testing

### Full Workflow Test

Here's a complete workflow to test end-to-end:

```bash
# 1. Setup
cd ~/colony-test
rm -rf .colony colony.yml
git worktree prune

# 2. Initialize
colony init

# 3. Customize config (optional)
# Edit colony.yml to your liking

# 4. Start agents
colony start --no-attach

# 5. Check status
colony status

# 6. Create some tasks
colony tasks create task-1 "Build API" "Create REST endpoints" --priority high
colony tasks create task-2 "Write Tests" "Unit tests for API" --priority medium

# 7. Assign tasks
colony tasks claim task-1 backend-1
colony tasks claim task-2 backend-1

# 8. Update progress
colony tasks progress task-1 50

# 9. Send a broadcast
colony broadcast "Team, please focus on API endpoints today"

# 10. Check messages
colony messages all

# 11. View logs
colony logs backend-1

# 12. Complete a task
colony tasks complete task-1

# 13. Check TUI
colony tui  # Press 'q' to exit

# 14. Stop agents
colony stop

# 15. Restart
colony start --no-attach

# 16. Verify tasks persisted
colony tasks list

# 17. Cleanup
colony destroy
```

**Expected results:**
- ✅ All commands succeed
- ✅ Tasks persist across restarts
- ✅ State is consistent
- ✅ No crashes or errors

---

## Troubleshooting

### Common Issues

**Issue: "tmux: command not found"**
- Install tmux: `sudo apt-get install tmux` (Ubuntu/Debian) or `brew install tmux` (macOS)

**Issue: "Not a git repository"**
- Run `git init` in your test directory
- Make at least one commit: `git commit --allow-empty -m "Init"`

**Issue: Tmux session won't start**
- Check if session already exists: `tmux ls`
- Kill old session: `tmux kill-session -t colony-test`
- Try again

**Issue: Worktree errors**
- List worktrees: `git worktree list`
- Prune orphaned worktrees: `git worktree prune`
- Manually remove: `git worktree remove .colony/worktrees/agent-name`

**Issue: Permission denied errors**
- Check file permissions: `ls -la .colony/`
- Fix permissions: `chmod -R u+w .colony/`

**Issue: Tasks not showing up**
- Check task directory: `ls .colony/tasks/*/`
- Verify JSON format: `cat .colony/tasks/pending/task-1.json | jq`

---

## Test Checklist

Use this checklist to track your manual testing progress:

### Basic Commands
- [ ] `colony --help` shows help
- [ ] `colony --version` shows version
- [ ] `colony init` creates config
- [ ] `colony start` spawns agents
- [ ] `colony status` shows agents
- [ ] `colony attach` connects to tmux
- [ ] `colony tui` launches dashboard
- [ ] `colony stop` stops agents
- [ ] `colony destroy` cleans up

### Task Management
- [ ] Create task
- [ ] List tasks
- [ ] Show task details
- [ ] Claim task
- [ ] Update progress
- [ ] Block task
- [ ] Unblock task
- [ ] Complete task
- [ ] Cancel task
- [ ] Delete task
- [ ] List agent tasks
- [ ] List claimable tasks

### Messaging
- [ ] Broadcast message
- [ ] List all messages
- [ ] List messages for agent

### Logs
- [ ] List all logs
- [ ] View specific agent log

### Error Handling
- [ ] Run outside git repo (should fail)
- [ ] Run without tmux (should fail gracefully)
- [ ] Invalid YAML config (should fail with clear error)
- [ ] Duplicate agent IDs (should fail)
- [ ] Non-existent agent operations (should fail)

### Edge Cases
- [ ] Re-initialize existing colony
- [ ] Start already running colony
- [ ] Stop already stopped agents
- [ ] Destroy and cancel
- [ ] Many agents (10+)
- [ ] Many tasks (100+)
- [ ] Large messages

---

## Tips for Manual Testing

1. **Use a test repository**: Don't test on your real projects!
2. **Reset between tests**: Run `colony destroy` to start fresh
3. **Check filesystem**: Use `ls`, `cat`, `tree` to verify state
4. **Watch tmux**: Use `tmux ls` and `tmux list-panes` to debug
5. **Check logs**: If something fails, check `.colony/logs/`
6. **Test incrementally**: Don't run all tests at once; go step-by-step
7. **Document issues**: Keep notes on bugs you find
8. **Try to break it**: Test with weird input, edge cases, and errors

---

## Reporting Issues

If you find bugs while testing, please report them with:

1. **Steps to reproduce**: Exact commands you ran
2. **Expected behavior**: What should have happened
3. **Actual behavior**: What actually happened
4. **Error messages**: Full error output
5. **Environment**: OS, tmux version, git version
6. **Config**: Your `colony.yml` (if relevant)

Example:
```
## Bug: Task deletion doesn't work

**Steps:**
1. colony init
2. colony tasks create task-1 "Test" "Description"
3. colony tasks delete task-1

**Expected:** Task deleted successfully
**Actual:** Error: "Permission denied"

**Error output:**
Error: Permission denied (os error 13)

**Environment:**
- OS: Ubuntu 22.04
- tmux: 3.2a
- git: 2.34.1
```

---

## Quick Reference

### Most Used Commands

```bash
# Setup
colony init

# Start/Stop
colony start
colony start --no-attach
colony stop
colony stop backend-1

# Monitor
colony status
colony tui
colony logs backend-1

# Tasks
colony tasks create <id> <title> <desc>
colony tasks list
colony tasks claim <task> <agent>
colony tasks complete <task>

# Messages
colony broadcast "message"
colony messages all

# Cleanup
colony destroy
```

---

## Next Steps

After manual testing:
1. File bugs for any issues found
2. Update this guide with new edge cases
3. Consider automating common test scenarios
4. Write integration tests for critical paths

Happy testing! 🚀
