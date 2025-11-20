//! State integration utilities for Colony agents

use crate::error::ColonyResult;
use std::path::{Path, PathBuf};

/// Create helper script for state operations that agents can use
pub fn create_state_helper_script(colony_root: &Path, agent_id: &str) -> ColonyResult<PathBuf> {
    let script_path = colony_root
        .join("projects")
        .join(agent_id)
        .join("colony_state.sh");

    // Shell-escape values to prevent injection
    let escaped_root = shell_escape_for_script(&colony_root.display().to_string());
    let escaped_agent = shell_escape_for_script(agent_id);

    let script_content = format!(
        r#"#!/bin/bash
# Colony State Helper Script for '{}'
# This script helps agents interact with shared state (tasks, workflows)

COLONY_ROOT='{}'
AGENT_ID='{}'
STATE_DIR="${{COLONY_ROOT}}/.colony/state"

# Find colony binary (check common locations)
COLONY_BIN=""
if command -v colony &> /dev/null; then
    COLONY_BIN="colony"
elif [ -x "${{COLONY_ROOT}}/target/debug/colony" ]; then
    COLONY_BIN="${{COLONY_ROOT}}/target/debug/colony"
elif [ -x "${{COLONY_ROOT}}/target/release/colony" ]; then
    COLONY_BIN="${{COLONY_ROOT}}/target/release/colony"
fi

if [ -z "$COLONY_BIN" ]; then
    echo "Error: colony binary not found"
    exit 1
fi

case "$1" in
    task)
        # Task operations: list, add, show, update, ready
        shift
        $COLONY_BIN state task "$@"
        ;;

    workflow)
        # Workflow operations: list, add, show, update
        shift
        $COLONY_BIN state workflow "$@"
        ;;

    memory)
        # Memory operations: add, search
        shift
        $COLONY_BIN state memory "$@"
        ;;

    sync)
        # Sync state (pull + push)
        $COLONY_BIN state sync
        ;;

    pull)
        # Pull latest state from remote
        $COLONY_BIN state pull
        ;;

    push)
        # Push local state to remote
        $COLONY_BIN state push
        ;;

    help|--help|-h|"")
        echo "Colony State Helper - Interact with shared state"
        echo ""
        echo "USAGE:"
        echo "  ./colony_state.sh <command> [args...]"
        echo ""
        echo "TASK COMMANDS:"
        echo "  task list                    List all tasks"
        echo "  task ready                   List ready-to-work tasks (no blockers)"
        echo "  task show <id>               Show task details"
        echo "  task add <title>             Create a new task"
        echo "  task update <id> <status>    Update task status"
        echo "                               Status: ready|blocked|in_progress|completed|cancelled"
        echo "  task assign <id> [agent-id]  Assign task to agent (defaults to current agent)"
        echo "  task block <id> <blocker>    Add blocker to task"
        echo ""
        echo "WORKFLOW COMMANDS:"
        echo "  workflow list                List all workflows"
        echo "  workflow show <id>           Show workflow details"
        echo "  workflow add <name>          Create a new workflow"
        echo "  workflow update <id> <status> Update workflow status"
        echo "                               Status: pending|running|completed|failed"
        echo ""
        echo "MEMORY COMMANDS:"
        echo "  memory add <type> <content>  Store a memory entry"
        echo "                               Type: context|learned|decision|note"
        echo "  memory search <query>        Search memory entries"
        echo ""
        echo "SYNC COMMANDS:"
        echo "  sync                         Pull and push state (full sync)"
        echo "  pull                         Pull latest state from remote"
        echo "  push                         Push local state to remote"
        echo ""
        echo "EXAMPLES:"
        echo "  # List ready tasks"
        echo "  ./colony_state.sh task ready"
        echo ""
        echo "  # Create a new task"
        echo "  ./colony_state.sh task add \"Implement feature X\""
        echo ""
        echo "  # Assign task to yourself"
        echo "  ./colony_state.sh task assign task-abc123"
        echo ""
        echo "  # Mark task as completed"
        echo "  ./colony_state.sh task update task-abc123 completed"
        echo ""
        echo "  # Store learned information"
        echo "  ./colony_state.sh memory add learned \"API rate limit is 100/hour\""
        echo ""
        echo "  # Sync state with team"
        echo "  ./colony_state.sh sync"
        ;;

    *)
        echo "Unknown command: $1"
        echo "Run './colony_state.sh help' for usage"
        exit 1
        ;;
esac
"#,
        agent_id, escaped_root, escaped_agent
    );

    // Write script
    std::fs::write(&script_path, script_content)?;

    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&script_path)?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&script_path, permissions)?;
    }

    Ok(script_path)
}

/// Create README for state system
pub fn create_state_readme(colony_root: &Path) -> ColonyResult<()> {
    let readme_path = colony_root.join(".colony").join("STATE_README.md");

    let readme_content = r#"# Colony Shared State System

This directory contains Colony's git-backed shared state, enabling agents to coordinate work across sessions.

## State Files

- `tasks.jsonl` - Work items with status, blockers, and assignments
- `workflows.jsonl` - Multi-step execution tracking
- `memory.jsonl` - Agent knowledge and learned context (optional)

## JSONL Format

Each file uses JSON Lines format (one JSON object per line):
- Append-only for git-friendly merging
- Each line is a complete state entry
- Git tracks full history of changes

## For Agents

Use the `colony_state.sh` helper script in your project directory:

```bash
# List ready tasks
./colony_state.sh task ready

# Create a task
./colony_state.sh task add "Implement authentication"

# Assign to yourself
./colony_state.sh task assign task-abc123

# Mark as completed
./colony_state.sh task update task-abc123 completed

# Store learned info
./colony_state.sh memory add learned "Database uses PostgreSQL 14"

# Sync with team
./colony_state.sh sync
```

## How It Works

1. **Git-Backed**: State stored in JSONL files tracked by git
2. **SQLite Cache**: Fast queries via local cache (`.colony/cache/state.db`)
3. **Auto-Sync**: Changes automatically committed/pushed (if configured)
4. **Distributed**: Each agent can work independently, git handles merging
5. **Hash-Based IDs**: Collision-free task/workflow IDs (Beads-inspired)

## Configuration

State system configured in `colony.yml`:

```yaml
shared_state:
  backend: git-backed      # git-backed or memory
  location: in-repo        # in-repo or external
  path: .colony/state      # State directory
  auto_commit: true        # Auto-commit changes
  auto_push: false         # Auto-push to remote
```

See `colony.example.yml` for more configuration options.

## Task Workflow

1. **Create**: Agent creates task with blockers if needed
2. **Ready**: Task becomes ready when blockers complete
3. **Assign**: Agent assigns task to themselves
4. **Work**: Agent works on task, updates status to `in_progress`
5. **Complete**: Agent marks task as `completed`
6. **Dependencies**: Other tasks blocked on this become ready

## Workflows

Multi-step processes with:
- Named steps with individual status
- Input/output tracking
- Agent assignment per step
- Overall workflow status

## Memory

Agents can store/retrieve:
- **Context**: Key-value pairs (API URLs, credentials paths, etc.)
- **Learned**: Discovered information during work
- **Decisions**: Important decisions and rationale
- **Notes**: General observations

## Best Practices

1. **Small Tasks**: Break work into independent, parallelizable tasks
2. **Clear Blockers**: Explicitly declare task dependencies
3. **Update Status**: Keep task status current for coordination
4. **Sync Often**: Pull before starting work, push after completing
5. **Meaningful Titles**: Task titles should be clear and specific
6. **Use Memory**: Store learned info for other agents
7. **Check Ready**: Use `task ready` to find available work

## External State (Optional)

For multi-project coordination:

```yaml
shared_state:
  location: external
  repository: git@github.com:org/colony-state.git
  project_id: my-project
  auto_push: true
```

All projects share the same state repository, each in their own namespace.

## Troubleshooting

**Cache out of sync?**
```bash
rm .colony/cache/state.db
# Cache will rebuild automatically
```

**Merge conflicts?**
```bash
cd .colony/state
git status
# Resolve conflicts manually (JSONL is append-only, usually easy)
```

**State not syncing?**
Check `colony.yml` configuration and git remote setup.

## Learn More

- Documentation: `docs/GIT-STATE-IMPLEMENTATION.md`
- Architecture: `docs/BEADS-INTEGRATION.md`
- Configuration: `colony.example.yml`
"#;

    // Create .colony directory if it doesn't exist
    std::fs::create_dir_all(colony_root.join(".colony"))?;

    std::fs::write(readme_path, readme_content)?;

    Ok(())
}

/// Shell-escape a string for safe inclusion in a bash script
fn shell_escape_for_script(s: &str) -> String {
    // For values embedded in script (inside single quotes in bash variables)
    // We need to escape single quotes by ending the quote, adding escaped quote, and starting quote again
    s.replace('\'', "'\"'\"'")
}
