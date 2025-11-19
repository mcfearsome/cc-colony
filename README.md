# Colony

Multi-agent orchestration for Claude Code on tmux.

Colony enables you to run multiple Claude Code agents in parallel, each in their own isolated tmux session, with proper state management and inter-agent communication.

## Features

- **Multi-Agent Orchestration**: Run multiple Claude Code instances in parallel
- **Tmux Integration**: Each agent runs in its own tmux pane for easy monitoring
- **Task Management**: Assign and track tasks across agents
- **Inter-Agent Messaging**: Broadcast messages and communicate between agents
- **Interactive TUI**: Monitor and control your colony with a terminal UI
- **Remote Control**: Connect to relay service for mobile/web control
- **Git Worktree Isolation**: Each agent works in its own git worktree
- **Shared Worktrees**: Multiple agents can collaborate in the same worktree
- **Per-Agent Environment Variables**: Configure environment variables for each agent
- **Per-Agent MCP Configuration**: Each agent can have its own MCP server setup
- **Custom Instructions**: Automatically inject specialized prompts when agents start

## Commands

### Core Commands
- `colony init` - Initialize a new colony configuration
- `colony start` - Start all agents in the colony
- `colony attach` - Attach to the tmux session
- `colony tui` - Launch the interactive TUI
- `colony status` - Show status of running agents
- `colony broadcast <message>` - Send a message to all agents
- `colony stop [agent_id]` - Stop one or all agents
- `colony logs [agent_id]` - View agent logs
- `colony destroy` - Destroy the colony and clean up resources
- `colony tasks` - Manage tasks (list, create, claim, etc.)
- `colony messages` - View messages between agents

### Authentication
- `colony auth login` - Authenticate with OAuth, API key, or Bedrock
- `colony auth status` - Show authentication status
- `colony auth logout` - Remove credentials
- `colony auth refresh` - Refresh authentication token

### Remote Control
- `colony relay connect` - Connect to relay service for mobile/web control
- `colony relay status` - Show relay connection status
- `colony relay disconnect` - Disconnect from relay service

## Building

```bash
cargo build --release
```

## Installation

```bash
cargo install --path .
```

## Usage

1. Initialize a colony in your project:
   ```bash
   colony init
   ```

2. Edit `colony.yml` to configure your agents

3. Start the colony:
   ```bash
   colony start
   ```

4. Monitor with the TUI:
   ```bash
   colony tui
   ```

## Configuration

Colony is configured via `colony.yml`. See the initialization output for basic configuration options, or check `colony.example.yml` for a comprehensive example with all features including per-agent MCP server configuration.

### Per-Agent MCP Servers

Each agent can have its own MCP (Model Context Protocol) server configuration:

```yaml
agents:
  - id: backend-1
    role: Backend Engineer
    focus: API development
    model: claude-opus-4-20250514
    mcp_servers:
      filesystem:
        command: npx
        args:
          - -y
          - "@modelcontextprotocol/server-filesystem"
          - /path/to/directory
      git:
        command: uvx
        args:
          - mcp-server-git
          - --repository
          - /path/to/repo
```

MCP servers configured in `colony.yml` will:
- Merge with any existing `.claude/settings.json` in the working directory
- Override repo-level settings with agent-specific configuration
- Be written to `.colony/projects/{agent-id}/.claude/settings.json`
- Be passed to Claude Code via the `--settings` flag

See `colony.example.yml` for more examples of MCP server configurations.

### Remote Control via Relay Service

Connect your colony to app.colony.sh for mobile and web control:

```bash
# Connect to relay service
colony relay connect

# You'll be prompted for a token from https://app.colony.sh
# Or provide it directly:
colony relay connect --token YOUR_TOKEN

# Check connection status
colony relay status

# Disconnect
colony relay disconnect
```

Once connected, you can:
- **Monitor** agents, tasks, and messages in real-time from your phone or web browser
- **Send commands** remotely (send messages, create tasks, stop/start agents)
- **Receive notifications** about colony activity
- **Control from anywhere** - perfect for checking on long-running tasks

The relay connection includes:
- **Auto-reconnect** with exponential backoff
- **Real-time state sync** every 5 seconds
- **Command feedback** - know if remote commands succeeded or failed
- **Secure WebSocket** connection with token authentication

**Supported remote commands:**
- Send messages to agents
- Broadcast messages to all agents
- Create tasks
- Stop agents
- Start agents
- Restart agents

Perfect for:
- Checking on builds while away from your desk
- Managing colonies from your phone
- Monitoring multiple colonies from a single dashboard

### Shared Worktrees

Multiple agents can work in the same worktree by specifying the same `worktree` name:

```yaml
agents:
  - id: reviewer-1
    role: Backend Reviewer
    focus: Review backend code
    worktree: shared-review  # Shares with reviewer-2

  - id: reviewer-2
    role: Frontend Reviewer
    focus: Review frontend code
    worktree: shared-review  # Shares with reviewer-1
```

Benefits of shared worktrees:
- Agents can collaborate on the same codebase
- Changes made by one agent are immediately visible to others
- Reduces disk space usage
- Useful for review, pair programming, or specialized focus areas

### Per-Agent Environment Variables

Set custom environment variables for each agent:

```yaml
agents:
  - id: test-agent
    role: Integration Tester
    focus: Run integration tests
    env:
      TEST_ENV: integration
      DATABASE_URL: postgresql://localhost:5432/test_db
      LOG_LEVEL: debug
```

Environment variables are:
- Exported before Claude Code starts
- Isolated to each agent's pane
- Shell-escaped for security
- Useful for API keys, feature flags, and per-agent configuration

### Custom Instructions

Add specialized instructions to each agent's startup prompt:

```yaml
agents:
  - id: security-auditor
    role: Security Auditor
    focus: Review code for security vulnerabilities
    instructions: |
      Your mission is to identify security vulnerabilities.

      Focus areas:
      - SQL injection vulnerabilities
      - XSS (Cross-Site Scripting) issues
      - Authentication and authorization flaws

      When you find an issue:
      1. Document the vulnerability
      2. Assess severity (Critical/High/Medium/Low)
      3. Provide a fix example
```

Custom instructions:
- Are automatically sent to Claude when the agent starts
- Appear after the standard colony prompt (role, focus, messaging)
- Support multi-line YAML strings with `|`
- Perfect for specialized behaviors, checklists, or workflows

## Skills

Colony includes built-in skills to help agents with common tasks:

### Colony Message Skill

The `colony-message` skill provides comprehensive guidance on using the inter-agent messaging system. Agents can invoke it to learn how to:
- Send messages to specific agents
- Broadcast to all agents
- Check for incoming messages
- Follow messaging best practices
- Coordinate work with other agents

**Usage:** Agents can refer to `.claude/skills/colony-message.md` or invoke the skill for messaging guidance.

**Example workflows covered:**
- Announcing work in progress
- Requesting code reviews
- Reporting bugs to the team
- Coordinating on shared resources
- Getting unblocked

## Dashboard

A live dashboard script is available at `scripts/colony_dashboard.sh` for real-time monitoring.

## License

MIT
