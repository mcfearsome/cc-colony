# Git-Backed State

Implementation details of the git-backed state system.

## Design

Based on Beads principles:
- Append-only JSONL logs
- SQLite query layer
- Git version control
- Hash-based IDs

## Storage

```
.colony/state/
├── tasks.jsonl
├── workflows.jsonl
├── memory.jsonl
└── query.db
```

See [BEADS Integration](../../BEADS-INTEGRATION.md) for full details.
