# Colony

Multi-agent orchestration for Claude Code on tmux.

Colony enables you to run multiple Claude Code agents in parallel, each in their own isolated tmux session, with proper state management and inter-agent communication.

## Features

- **Multi-Agent Orchestration**: Run multiple Claude Code instances in parallel
- **Tmux Integration**: Each agent runs in its own tmux session for easy monitoring
- **Task Management**: Assign and track tasks across agents
- **Inter-Agent Messaging**: Broadcast messages and communicate between agents
- **Interactive TUI**: Monitor and control your colony with a terminal UI
- **Git Worktree Isolation**: Each agent works in its own git worktree

## Commands

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

Colony is configured via `colony.yml`. See the initialization output for configuration options.

## Dashboard

A live dashboard script is available at `scripts/colony_dashboard.sh` for real-time monitoring.

## License

MIT
