# State Management

Colony uses a git-backed state system for reliable agent coordination.

## Architecture

- **Backend**: Git (JSONL + SQLite)
- **Schema**: Beads-inspired design
- **Sync**: Push/pull model

## State Components

### Tasks
- Distributed task queue
- Dependency tracking
- Status management

### Workflows
- Multi-step processes
- State transitions
- History tracking

### Memory
- Shared context
- Learned insights
- Decision history

See [Git-Backed State](../advanced/git-state.md) for implementation details.
