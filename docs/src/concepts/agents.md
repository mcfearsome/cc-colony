# Agents

Agents are individual Claude Code instances running in isolated environments.

## Overview

Each agent in Colony:
- Runs in its own tmux pane
- Has its own git worktree
- Can use specialized templates
- Shares state with other agents
- Has a unique ID and configuration

## Agent Lifecycle

1. **Configuration** - Defined in colony.yml
2. **Initialization** - Worktree and environment setup
3. **Start** - Claude Code instance launched
4. **Running** - Agent performs tasks
5. **Stop** - Graceful shutdown

## Agent Isolation

Agents are isolated through:
- Separate git worktrees (no conflicts)
- Individual tmux panes (process isolation)
- Unique working directories
- Independent MCP server instances

## Communication

Agents communicate through:
- Shared state system
- Task queue
- Message passing
- Shared memory

See [State Management](./state.md) for details.
