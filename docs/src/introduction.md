# Introduction

**Colony** is a multi-agent orchestration system for Claude Code running on tmux. It enables you to run multiple Claude Code instances in parallel with proper isolation, state management, and coordination.

## What is Colony?

Colony provides infrastructure for managing multiple AI agents working together on complex software engineering tasks. Each agent runs in its own isolated environment (tmux pane) with dedicated resources, while a shared state system enables coordination and collaboration.

## Key Features

### Multi-Agent Orchestration
- Run multiple Claude Code instances simultaneously
- Each agent operates in an isolated tmux pane
- Automatic session management and health monitoring
- Interactive TUI for real-time monitoring

### Shared State Management
- Git-backed state storage (JSONL + SQLite)
- Distributed task queues with dependency resolution
- Shared memory and context
- Automatic state synchronization

### Workflow Orchestration
- Define complex multi-agent workflows in YAML
- DAG-based dependency resolution
- Retry policies and error handling
- Workflow history and status tracking

### Agent Templates
- Reusable agent configurations
- Built-in template library (code-reviewer, security-auditor, test-engineer, etc.)
- Custom template support
- Role-based specialization

### Plugin System
- Extensible architecture
- Backend, UI, and Tool plugins
- Discovery and lifecycle management
- Enable/disable plugin management

### Enhanced Logging & Observability
- Structured logging with filtering
- JSON log output support
- Real-time TUI with metrics panel
- Health checks and status monitoring

## Use Cases

- **Parallel Development**: Multiple agents working on different features simultaneously
- **Code Review**: Specialized review agents analyzing different aspects of code
- **Testing & QA**: Automated testing agents running comprehensive test suites
- **Documentation**: Documentation agents maintaining technical docs
- **Security Audits**: Security-focused agents scanning for vulnerabilities
- **Data Analysis**: Analyst agents processing and visualizing data

## Architecture

Colony uses a modular architecture with:
- **Tmux Integration**: Process isolation and session management
- **Git-Backed State**: Reliable, versioned state storage
- **YAML Configuration**: Human-readable configuration files
- **Rust Implementation**: Fast, safe, and reliable execution

## Quick Example

```bash
# Initialize a new colony
colony init

# Start all agents
colony start

# Monitor with TUI
colony tui

# View agent status
colony status

# Check logs
colony logs --level info
```

## Next Steps

- [Installation](./getting-started/installation.md) - Install Colony
- [Quick Start](./getting-started/quick-start.md) - Get started in 5 minutes
- [Core Concepts](./concepts/agents.md) - Understand the fundamentals
- [CLI Reference](./cli/overview.md) - Complete command reference
