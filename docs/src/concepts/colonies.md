# Colonies

A colony is a collection of agents working together on a project.

## What is a Colony?

A colony consists of:
- Multiple agent instances
- Shared state system
- Tmux session for isolation
- Configuration file (colony.yml)

## Colony Structure

```
my-project/
├── colony.yml          # Configuration
├── .colony/            # Colony data
│   ├── state/          # Shared state
│   ├── logs/           # Agent logs
│   ├── templates/      # Templates
│   └── plugins/        # Plugins
└── .git-worktrees/     # Agent worktrees
```

## Benefits

- Parallel execution
- Specialization
- Coordination
- State sharing
