# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Colony is a multi-agent orchestration system for Claude Code that runs multiple Claude instances in parallel using tmux for session management. Each agent runs in its own isolated environment with proper state management and inter-agent communication capabilities.

## Build and Test Commands

```bash
# Build the project
cargo build --release

# Install locally
cargo install --path .

# Run development build
cargo run -- <command>
```

## Interactive TUI

The TUI (`colony tui`) has 5 tabs:

1. **Agents** - Monitor agent status and activity
2. **Tasks** - View and manage the task queue
3. **Messages** - View inter-agent messages
4. **Compose** - Send messages with multiline input and recipient selection
5. **Help** - Keyboard shortcuts and documentation

### Compose Tab Workflow

The Compose tab provides a dedicated interface for sending messages:

1. **Message Field** (focus starts here):
   - Multiline text input for message content
   - Press `Enter` to add newlines
   - Type naturally without restrictions

2. **Recipient Selector** (press `Tab` to switch):
   - Visual list of all recipients ("all" + agent IDs)
   - Navigate with `↑/↓` arrow keys
   - Selected recipient highlighted in yellow
   - Shows "(broadcast to all agents)" hint for "all" option

3. **Sending**:
   - Press `Enter` when in recipient selector to send
   - Message automatically includes sender context (directory + git branch)
   - Fields clear after successful send
   - Press `Esc` to clear message without sending

## High-Level Architecture

### Core Orchestration Flow

The system follows this architecture pattern:

1. **ColonyController** (`src/colony/controller.rs`) - Central orchestrator that:
   - Initializes agents from `colony.yml` configuration
   - Creates Git worktrees for each agent (or uses custom directories)
   - Manages the `.colony/` state directory structure
   - Coordinates agent lifecycle

2. **Tmux Integration** (`src/colony/tmux.rs`) - Session management:
   - Creates a single tmux session with multiple panes (one per agent + optional executor)
   - Each pane runs Claude Code with agent-specific configuration
   - Handles environment variable isolation per pane
   - Manages MCP server configuration through `.claude/settings.json` files

3. **Agent State Management** (`src/colony/agent.rs`) - Each agent has:
   - Working directory: Either a Git worktree in `.colony/worktrees/{name}` or a custom directory
   - Project directory: `.colony/projects/{agent-id}` for agent-specific config
   - Log file: `.colony/logs/{agent-id}.log`
   - MCP settings: `.colony/projects/{agent-id}/.claude/settings.json` (merged from colony.yml and repo settings)

4. **Inter-Agent Communication** (`src/colony/messaging.rs`) - Message queue system:
   - Messages stored in `.colony/messages/{recipient-id}/`
   - Broadcast messages in `.colony/messages/broadcast/`
   - Typed messages: Info, Task, Question, Answer, Completed, Error
   - **Automatic Context**: Every message includes sender's working directory and git branch
   - Agents check messages via the colony-message skill

5. **Task Management** (`src/colony/tasks/`) - Shared task queue:
   - Tasks stored in `.colony/tasks.json`
   - Task states: Pending, Claimed, InProgress, Blocked, Completed, Cancelled
   - Agents can claim, update, and complete tasks
   - Supports priority levels and progress tracking

### Worktree Sharing Mechanism

Multiple agents can share the same Git worktree by specifying the same `worktree` name in `colony.yml`. The controller deduplicates worktree creation, so only one worktree is created per unique name. This enables:
- Collaborative work on the same codebase
- Reduced disk space usage
- Specialized review or parallel implementation workflows

### Claude Configuration Inheritance in Worktrees

When a worktree is created (`src/colony/worktree.rs:86-114`), the `.claude` directory from the main repository is **symlinked** into the worktree:

```
Main repo:     /path/to/repo/.claude/
                    ↓ (symlink)
Worktree:      .colony/worktrees/agent-1/.claude/ → /path/to/repo/.claude/
```

**Why symlinking:**
- `.claude` is typically gitignored, so worktrees don't inherit it via git
- Symlink ensures agents get all MCP servers, hooks, and settings from main repo
- Changes to main repo's `.claude` affect all agents immediately
- No duplication of configuration files

**Setting priority order** (when `--settings` flag is NOT used):
1. Worktree `.claude/settings.json` (symlinked from main repo)
2. Global `~/.claude/config.json` (only if `--setting-sources` includes global)

**Setting priority order** (when `--mcp-config` flag IS used):
1. Specified MCP config file (`.colony/projects/{agent-id}/.claude/settings.json`)
2. Other MCP sources ignored due to `--strict-mcp-config`
3. Other settings (hooks, statusLine) still loaded from local/user config

### MCP Configuration Merging

**IMPORTANT**: MCP servers are configured PER-AGENT in `colony.yml`. Agents without `mcp_servers` configuration will inherit settings from the symlinked `.claude/settings.json` in their worktree, which may include unexpected MCP servers.

#### When Agent HAS `mcp_servers` in `colony.yml`:

Command includes MCP-specific flags:
```bash
claude --mcp-config .colony/projects/{agent-id}/.claude/settings.json --strict-mcp-config
```

Merge order (highest priority first):
1. Agent-specific `mcp_servers` from `colony.yml`
2. Existing `.claude/settings.json` in main repo (via worktree symlink)
3. Merged result written to `.colony/projects/{agent-id}/.claude/settings.json`
4. Only this file is used for MCP servers (`--strict-mcp-config` prevents user config MCP servers)

**Critical**: We use `--mcp-config` with `--strict-mcp-config` instead of `--settings` because:
- `--mcp-config` specifically loads MCP server configuration
- `--strict-mcp-config` prevents MCP servers from `~/.claude.json` (user config) from being loaded
- This ensures agents get ONLY the MCP servers specified in colony.yml
- Other settings (hooks, statusLine, etc.) still come from local/user config as normal

#### When Agent DOES NOT HAVE `mcp_servers` in `colony.yml`:

Command uses `--setting-sources local` but NO `--settings` flag:
```bash
claude --setting-sources local --add-dir {worktree}
```

Claude looks for settings in:
1. Worktree `.claude/settings.json` (symlinked from main repo)
2. No merging occurs - uses whatever is in that file
3. May include MCP servers you don't expect (check main repo's `.claude/settings.json`)

**Recommendation**: If you want specific MCP servers for an agent, explicitly configure them in `colony.yml` even if they're the same as the main repo. This ensures predictable configuration and proper merging.

## Key Implementation Details

### State Directory Structure (`.colony/`)

```
.colony/
├── worktrees/          # Git worktrees (one per unique worktree name)
│   ├── agent-1/
│   └── shared-review/  # Multiple agents can share this
├── projects/           # Per-agent project directories
│   ├── agent-1/
│   │   └── .claude/
│   │       └── settings.json  # Merged MCP config
│   └── agent-2/
├── logs/               # Agent logs
│   ├── agent-1.log
│   └── agent-2.log
├── messages/           # Inter-agent message queue
│   ├── agent-1/        # Messages for agent-1
│   ├── agent-2/
│   └── broadcast/      # Broadcast messages
└── tasks.json          # Shared task queue
```

### Agent Initialization Sequence

1. Controller reads `colony.yml`
2. Creates `.colony/` directories
3. For each agent:
   - Creates or reuses Git worktree (unless using custom directory)
   - Creates project directory
   - Merges MCP configuration
   - Creates messaging script in project directory and symlinks to worktree
   - Generates startup prompt with role, focus, and messaging instructions
4. Tmux session created with layout based on agent count
5. Each pane exports environment variables and starts Claude Code with:
   - `--model` for the specified Claude model
   - `--settings` pointing to merged MCP config
   - Initial prompt piped via stdin

### Messaging Script Path Resolution

The `colony_message.sh` script is stored in `.colony/projects/{agent-id}/colony_message.sh` with hardcoded paths to the colony root. To make it accessible from the agent's working directory in `.colony/worktrees/{name}/`, two symlinks are created:

- **Script location**: `.colony/projects/{agent-id}/colony_message.sh` (contains `COLONY_ROOT` and `AGENT_ID` variables)
- **Generic symlink**: `.colony/worktrees/{name}/colony_message.sh` → `../../projects/{agent-id}/colony_message.sh`
- **Agent-specific symlink**: `.colony/worktrees/{name}/colony_message_{agent-id}.sh` → `../../projects/{agent-id}/colony_message.sh`

**For single-agent worktrees**: Agents use `./colony_message.sh` for convenience.

**For shared worktrees**: Agents must use `./colony_message_{agent-id}.sh` to ensure they're using their own script with the correct agent ID. The generic `./colony_message.sh` symlink will point to whichever agent was initialized last.

### Message Context Fields

Messages automatically capture and include context to help recipients understand where work is happening:

```json
{
  "id": "agent-1-1234567890",
  "from": "agent-1",
  "to": "mcp-executor",
  "content": "Execute MCP workflow to analyze user data",
  "timestamp": "2025-01-13T12:00:00Z",
  "message_type": "task",
  "project_dir": "/Users/user/project/.colony/worktrees/agent-1",
  "git_branch": "feature/user-analytics"
}
```

**Fields captured automatically by the shell script:**
- `project_dir`: Sender's current working directory (`$PWD`)
- `git_branch`: Current git branch (if in a git repository)

**Critical use cases:**
- **MCP Executor**: Uses `project_dir` to know where to execute scripts and find files
- **Code Review**: Recipients know which worktree/branch to examine
- **File References**: Relative paths in message content are relative to `project_dir`
- **Collaboration**: Clarifies which workspace contains relevant changes

### Custom Instructions

The `instructions` field in agent config is appended to the generated prompt after the standard colony setup (role, focus, messaging system). This allows specialized behaviors without modifying the core prompt generation.

## Error Handling Patterns

- All operations return `ColonyResult<T>` which is `Result<T, ColonyError>`
- Custom error types in `src/error.rs` with `thiserror` derive
- Tmux commands wrap errors with context about which agent/operation failed
- Git worktree operations validate repository state before attempting operations
