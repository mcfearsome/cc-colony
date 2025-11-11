# Forge CLI

A powerful command-line tool for managing Claude Desktop MCP (Model Context Protocol) server configurations.

## Features

- **Easy Configuration Management**: Auto-detect and manage Claude Desktop configs across all platforms
- **MCP Server Discovery**: Search and browse available MCP servers from useforge.cc
- **One-Command Installation**: Install and remove MCP servers with simple commands
- **Bundle Support**: Apply curated collections of MCP servers for specific workflows
- **Cloud Sync**: Pull latest recommendations and updates from the cloud
- **Health Checks**: Validate configurations and detect conflicts with `forge doctor`
- **Backup & Restore**: Export and import your configurations safely
- **Swarm Orchestration**: Run multiple Claude Code agents in parallel with task and message management
- **Beautiful CLI**: Colored output, progress indicators, and interactive prompts

## Installation

### From Source

```bash
git clone https://github.com/yourusername/cli.useforge.cc.git
cd cli.useforge.cc
cargo build --release
sudo cp target/release/forge /usr/local/bin/
```

### Prerequisites

- Rust 1.70 or higher
- Claude Desktop installed on your system

## Quick Start

```bash
# Initialize forge in your current directory
forge init

# Authenticate with useforge.cc
forge login

# Search for available MCP servers
forge search

# Add an MCP server
forge add server-name

# List installed servers
forge list

# Check for issues
forge doctor
```

## Commands

### `forge init`
Set up forge in the current directory. Creates necessary configuration files and detects your Claude Desktop installation.

```bash
forge init
```

### `forge login`
Authenticate with the useforge.cc API to access the MCP server registry.

```bash
forge login
```

### `forge list`
Display all currently installed MCP servers in your Claude Desktop configuration.

```bash
forge list
```

### `forge search [query]`
Search for available MCP servers. Optionally provide a search query to filter results.

```bash
# Search all servers
forge search

# Search for specific servers
forge search filesystem
```

### `forge add <server-name>`
Install an MCP server to your Claude Desktop configuration.

```bash
forge add filesystem-server
```

The command will:
- Fetch server details from the registry
- Show you what will be installed
- Ask for confirmation
- Update your Claude Desktop config
- Remind you to restart Claude Desktop

### `forge remove <server-name>`
Uninstall an MCP server from your configuration.

```bash
forge remove filesystem-server
```

### `forge bundle list`
Show available bundles (curated collections of MCP servers for specific use cases).

```bash
forge bundle list
```

### `forge bundle activate <name>`
Install all servers from a bundle.

```bash
forge bundle activate web-development
```

### `forge sync`
Pull latest recommendations and available updates from the cloud.

```bash
forge sync
```

This will:
- Show recommended servers based on your usage
- Display available updates for installed servers
- Optionally install updates

### `forge doctor`
Run diagnostics to check for configuration issues, conflicts, and problems.

```bash
forge doctor
```

Checks include:
- Claude Desktop config file exists and is valid JSON
- Server configurations are properly formatted
- Command paths are valid
- No duplicate or conflicting servers
- Forge configuration is healthy

### `forge export [--output <file>]`
Export your current configuration as a backup.

```bash
# Export with auto-generated filename
forge export

# Export to specific file
forge export --output my-config-backup.json
```

### `forge import <file>`
Import a configuration from a backup file.

```bash
forge import my-config-backup.json
```

**Warning**: This will replace your current configuration!

## Forge Swarm - Multi-Agent Orchestration

Forge Swarm enables you to run multiple Claude Code instances in parallel, each working on different tasks with coordination through messaging and task queues.

### Swarm Lifecycle Commands

#### `forge swarm init`
Initialize a new swarm configuration in the current directory.

```bash
forge swarm init
```

This creates a `swarm.yml` configuration file and `.forge-swarm/` directory structure for agent coordination.

#### `forge swarm start`
Start all agents defined in your swarm configuration (requires tmux).

```bash
forge swarm start
```

This will:
- Create git worktrees for each agent (proper isolation)
- Launch Claude Code instances in separate tmux windows
- Set up messaging infrastructure
- Initialize task queues

#### `forge swarm attach`
Attach to the tmux session to watch agents work in real-time.

```bash
forge swarm attach
```

Use `Ctrl+B` then `D` to detach without stopping agents.

#### `forge swarm status`
Show the current status of all running agents.

```bash
forge swarm status
```

Displays:
- Agent IDs and their status (running/stopped)
- Active tasks per agent
- Last activity timestamps

#### `forge swarm stop [agent-id]`
Stop one or all agents in the swarm.

```bash
# Stop all agents
forge swarm stop

# Stop a specific agent
forge swarm stop backend-1
```

#### `forge swarm destroy`
Completely destroy the swarm and clean up all resources.

```bash
forge swarm destroy
```

**Warning**: This removes all worktrees, task data, and messages!

### Message Management

Agents can communicate with each other through a persistent message queue.

#### `forge swarm messages list <agent-id>`
View all messages for a specific agent.

```bash
forge swarm messages list backend-1
```

Shows:
- Direct messages to the agent
- Broadcast messages visible to all agents
- Message types (INFO, TASK, QUESTION, ANSWER, COMPLETED, ERROR)
- Timestamps and sender information

#### `forge swarm messages all`
View all messages in the entire swarm system.

```bash
forge swarm messages all
```

Useful for:
- Debugging communication issues
- Understanding swarm coordination
- Auditing agent interactions

#### `forge swarm broadcast <message>`
Send a broadcast message to all agents.

```bash
forge swarm broadcast "API endpoints are now live - ready for integration testing"
```

### Task Management

Coordinate work across agents with a sophisticated task queue system.

#### `forge swarm tasks list [--status <status>] [--compact]`
List all tasks in the swarm.

```bash
# List all tasks (full view with statistics)
forge swarm tasks list

# Filter by status
forge swarm tasks list --status pending
forge swarm tasks list --status in_progress

# Use compact view
forge swarm tasks list --compact
```

Status filters: `pending`, `claimed`, `in_progress`, `blocked`, `completed`, `cancelled`

#### `forge swarm tasks show <task-id>`
Show detailed information for a specific task.

```bash
forge swarm tasks show implement-auth
```

Displays:
- Task status and priority
- Assigned agent and progress
- Dependencies and blockers
- Timestamps (created, claimed, started, completed)

#### `forge swarm tasks create <task-id> <title> <description> [--assigned-to <agent>] [--priority <priority>]`
Create a new task in the queue.

```bash
# Create a task for any agent to claim
forge swarm tasks create auth-impl "Implement authentication" "Add JWT-based auth to API"

# Assign to specific agent
forge swarm tasks create db-setup "Database setup" "Initialize PostgreSQL" --assigned-to backend-1

# Set priority
forge swarm tasks create security-fix "Fix XSS vulnerability" "Sanitize user inputs" --priority critical
```

Priority levels: `low`, `medium` (default), `high`, `critical`

#### `forge swarm tasks claim <task-id> <agent-id>`
Claim a task for an agent to work on.

```bash
forge swarm tasks claim auth-impl backend-1
```

Validates that:
- Task is in pending status
- All dependencies are completed
- Agent is authorized to claim the task

#### `forge swarm tasks progress <task-id> <progress>`
Update the progress percentage (0-100) for a task.

```bash
forge swarm tasks progress auth-impl 75
```

#### `forge swarm tasks block <task-id> <reason>`
Mark a task as blocked with a reason.

```bash
forge swarm tasks block auth-impl "Waiting for API key from security team"
```

#### `forge swarm tasks unblock <task-id>`
Clear blockers and resume work on a blocked task.

```bash
forge swarm tasks unblock auth-impl
```

#### `forge swarm tasks complete <task-id>`
Mark a task as completed.

```bash
forge swarm tasks complete auth-impl
```

Automatically sets progress to 100% and updates dependent tasks.

#### `forge swarm tasks cancel <task-id>`
Cancel a task (cannot cancel completed tasks).

```bash
forge swarm tasks cancel deprecated-feature
```

#### `forge swarm tasks delete <task-id>`
Permanently delete a task (with confirmation prompt).

```bash
forge swarm tasks delete old-task
```

#### `forge swarm tasks agent <agent-id>`
List all tasks assigned to or claimed by a specific agent.

```bash
forge swarm tasks agent backend-1
```

Shows tasks in any status that belong to the agent.

#### `forge swarm tasks claimable <agent-id>`
Show tasks that an agent can currently claim.

```bash
forge swarm tasks claimable backend-1
```

Filters for:
- Tasks in pending status
- Tasks with no unfulfilled dependencies
- Tasks assigned to "auto" or the specific agent

#### `forge swarm logs [agent-id]`
View logs for one or all agents.

```bash
# List all agent logs
forge swarm logs

# View specific agent's log
forge swarm logs backend-1
```

### Swarm Configuration Example

**swarm.yml**:
```yaml
name: fullstack-app
description: Full-stack application development swarm

agents:
  - id: backend-1
    branch_prefix: feature/backend
    description: Backend API development

  - id: frontend-1
    branch_prefix: feature/frontend
    description: React frontend development

  - id: devops-1
    branch_prefix: feature/infra
    description: Infrastructure and deployment

initial_tasks:
  - id: setup-database
    title: "Set up PostgreSQL database"
    description: "Create schema and initial migrations"
    assigned_to: backend-1
    priority: high

  - id: api-endpoints
    title: "Implement REST API endpoints"
    description: "Create CRUD endpoints for all resources"
    assigned_to: backend-1
    dependencies: [setup-database]

  - id: frontend-components
    title: "Build React components"
    description: "Create reusable UI components"
    assigned_to: frontend-1

  - id: docker-setup
    title: "Create Docker configuration"
    description: "Set up multi-container Docker environment"
    assigned_to: devops-1
```

### Swarm Workflow Example

```bash
# 1. Initialize swarm
forge swarm init

# 2. Edit swarm.yml to define agents and tasks

# 3. Start the swarm
forge swarm start

# 4. Monitor progress
forge swarm status
forge swarm tasks list

# 5. Coordinate work
forge swarm broadcast "Database schema finalized - ready for API development"

# 6. View agent-specific work
forge swarm tasks agent backend-1
forge swarm messages list backend-1

# 7. Manage blockers
forge swarm tasks block api-endpoints "Need frontend API contract review"

# 8. Watch agents work
forge swarm attach

# 9. Review and complete
forge swarm tasks complete api-endpoints

# 10. Clean up when done
forge swarm stop
forge swarm destroy
```

### Agents Messaging from Inside

Agents can send messages to each other using the helper script created in each worktree:

```bash
# Inside an agent's worktree
./swarm_message.sh send backend-1 "API endpoints ready for testing"
./swarm_message.sh send all "Deployment to staging complete"
./swarm_message.sh read
./swarm_message.sh list-agents
```

## Configuration

### Claude Desktop Config Location

Forge automatically detects your Claude Desktop configuration based on your OS:

- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%/Claude/claude_desktop_config.json`
- **Linux**: `~/.config/Claude/claude_desktop_config.json`

### Forge Config

Forge stores its own configuration in `~/.forge/config.json`, including:
- API authentication token
- API base URL
- Cache settings

## MCP Server Format

MCP servers in your Claude Desktop config follow this structure:

```json
{
  "mcpServers": {
    "server-name": {
      "command": "node",
      "args": ["/path/to/server/index.js"],
      "env": {
        "API_KEY": "your-api-key"
      }
    }
  }
}
```

Forge manages this for you automatically!

## Examples

### Setting up a development environment

```bash
# Initialize forge
forge init

# Login to useforge.cc
forge login

# Search for development tools
forge search development

# Install a bundle of dev tools
forge bundle activate full-stack-dev

# Check everything is working
forge doctor
```

### Backup before making changes

```bash
# Export current config
forge export --output backup-$(date +%Y%m%d).json

# Make changes...
forge add new-server

# If something goes wrong
forge import backup-20240115.json
```

### Keep servers up to date

```bash
# Sync with cloud and install updates
forge sync
```

## Development

### Building from Source

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

### Project Structure

```
src/
├── main.rs              # CLI entry point with clap
├── api.rs               # API client for useforge.cc
├── config.rs            # Config management (Claude & Forge)
├── error.rs             # Error types and handling
├── utils.rs             # Utilities (colored output, prompts)
├── bundle/              # Bundle command implementations
│   ├── init.rs
│   ├── login.rs
│   ├── list.rs
│   ├── search.rs
│   └── ...
└── swarm/               # Swarm orchestration system
    ├── mod.rs           # Module exports
    ├── agent.rs         # Agent definitions and status
    ├── config.rs        # Swarm configuration (swarm.yml)
    ├── controller.rs    # Swarm lifecycle management
    ├── worktree.rs      # Git worktree isolation
    ├── tmux.rs          # Tmux session management
    ├── messaging.rs     # Inter-agent messaging
    ├── messages_cmd.rs  # Message CLI commands
    ├── tasks/           # Task queue system
    │   ├── mod.rs       # Task definitions
    │   ├── queue.rs     # Task queue management
    │   └── board.rs     # Task board rendering
    ├── tasks_cmd.rs     # Task CLI commands
    ├── init.rs          # Swarm initialization
    ├── start.rs         # Start agents
    ├── stop.rs          # Stop agents
    ├── status.rs        # Status reporting
    ├── attach.rs        # Attach to tmux
    ├── broadcast.rs     # Broadcast messages
    ├── logs.rs          # View agent logs
    └── destroy.rs       # Cleanup and teardown
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE file for details

## Support

- Documentation: [useforge.cc/docs](https://useforge.cc/docs)
- Issues: [GitHub Issues](https://github.com/yourusername/cli.useforge.cc/issues)
- API: [useforge.cc/api](https://useforge.cc/api)

## Troubleshooting

### "Claude Desktop not found"

Make sure Claude Desktop is installed on your system. Forge looks for the config file in the standard locations for your OS.

### "Not authenticated"

Run `forge login` to authenticate with the useforge.cc API before using commands that require authentication.

### "Permission denied"

On macOS/Linux, you may need to use `sudo` when installing to `/usr/local/bin/`.

### Changes not taking effect

Remember to restart Claude Desktop after making configuration changes with forge.

### Swarm: "tmux not found"

Swarm orchestration requires tmux for managing multiple agent sessions. Install it:

```bash
# macOS
brew install tmux

# Ubuntu/Debian
sudo apt-get install tmux

# Fedora
sudo dnf install tmux
```

### Swarm: Agents not responding

If agents appear stuck:
1. Check agent status: `forge swarm status`
2. View agent logs: `forge swarm logs <agent-id>`
3. Attach to session to see output: `forge swarm attach`
4. Check for blocked tasks: `forge swarm tasks list --status blocked`

### Swarm: Git worktree conflicts

If you encounter worktree issues:
```bash
# Clean up manually
git worktree prune
forge swarm destroy
forge swarm init
```
