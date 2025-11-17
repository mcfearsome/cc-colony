# Git-Backed State Implementation Plan

## Overview

This document outlines the implementation plan for git-backed state storage in Colony, inspired by [Beads](https://github.com/steveyegge/beads).

## Architecture Decision: In-Repository State

**Location**: `.colony/state/` within the project repository (default)

**Why in-repo?**
- State is project-specific (tasks relate to this codebase)
- Natural git integration (state commits with code)
- Branch isolation (different branches = different state)
- Zero setup (clone includes state)
- Familiar to developers (just files in git)

## Directory Structure

```
.colony/
├── state/                      # Git-tracked persistent state
│   ├── tasks.jsonl            # Task queue
│   ├── workflows.jsonl        # Workflow state
│   ├── messages.jsonl         # Archived messages
│   └── memory/
│       ├── coordinator.jsonl  # Per-agent memory
│       └── worker-1.jsonl
│
├── cache/                     # Git-ignored (in .gitignore)
│   ├── state.db              # SQLite cache for fast queries
│   └── .gitkeep
│
├── projects/                  # Agent Claude Code directories
│   ├── coordinator/
│   └── worker-1/
│
└── worktrees/                # Git worktrees for agents
    ├── coordinator/
    └── worker-1/
```

## File Formats

### Tasks (`.colony/state/tasks.jsonl`)

Each line is a JSON object representing one task:

```jsonl
{"id":"task-a1b2c3","title":"Add user authentication","status":"ready","created":"2024-11-17T14:30:00Z","assigned":null,"blockers":[],"metadata":{"priority":"high","tags":["auth","security"]}}
{"id":"task-d4e5f6","title":"Write auth tests","status":"blocked","created":"2024-11-17T14:31:00Z","assigned":null,"blockers":["task-a1b2c3"],"metadata":{"priority":"medium","tags":["testing"]}}
{"id":"task-g7h8i9","title":"Update documentation","status":"in_progress","created":"2024-11-17T14:32:00Z","assigned":"coordinator","blockers":[],"metadata":{"priority":"low","tags":["docs"]}}
{"id":"task-j0k1l2","title":"Deploy to production","status":"completed","created":"2024-11-17T14:00:00Z","assigned":"worker-1","blockers":[],"completed":"2024-11-17T15:30:00Z","metadata":{"priority":"high","tags":["deployment"]}}
```

**Schema:**
```rust
pub struct Task {
    pub id: String,           // "task-a1b2c3"
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,   // ready, blocked, in_progress, completed
    pub created: DateTime<Utc>,
    pub assigned: Option<String>,  // Agent ID
    pub blockers: Vec<String>,     // Task IDs
    pub completed: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}
```

### Workflows (`.colony/state/workflows.jsonl`)

```jsonl
{"id":"wf-1a2b3c","name":"data-pipeline","status":"running","started":"2024-11-17T14:00:00Z","current_step":"process","steps":{"fetch":{"status":"completed","output":"raw_data.json"},"process":{"status":"in_progress","started":"2024-11-17T14:15:00Z"}}}
```

### Agent Memory (`.colony/state/memory/{agent_id}.jsonl`)

```jsonl
{"timestamp":"2024-11-17T14:30:00Z","type":"context","key":"last_checkpoint","value":"step_5_complete"}
{"timestamp":"2024-11-17T14:31:00Z","type":"learned","content":"User authentication requires JWT tokens, not sessions"}
{"timestamp":"2024-11-17T14:32:00Z","type":"todo","content":"Need to refactor auth module before adding new features"}
```

## SQLite Cache Schema

**Purpose**: Fast queries without parsing JSONL every time

```sql
-- .colony/cache/state.db

CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL,
    created TEXT NOT NULL,
    assigned TEXT,
    blockers TEXT,  -- JSON array
    completed TEXT,
    metadata TEXT   -- JSON object
);

CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_assigned ON tasks(assigned);

CREATE TABLE workflows (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    status TEXT NOT NULL,
    started TEXT NOT NULL,
    current_step TEXT,
    steps TEXT  -- JSON object
);

CREATE TABLE agent_memory (
    agent_id TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    type TEXT NOT NULL,
    key TEXT,
    value TEXT,
    content TEXT,
    PRIMARY KEY (agent_id, timestamp)
);

-- Metadata table to track JSONL file timestamps
CREATE TABLE _meta (
    file TEXT PRIMARY KEY,
    last_import TEXT NOT NULL,  -- ISO timestamp
    checksum TEXT NOT NULL       -- File hash
);
```

## Implementation Phases

### Phase 1: Core Infrastructure (Week 1-2)

**Files to Create:**
```
src/colony/state/
├── mod.rs              # Public API
├── backend.rs          # GitBackedState struct
├── cache.rs            # SQLite cache management
├── jsonl.rs            # JSONL read/write
├── sync.rs             # Git commit/pull/push
└── schema.rs           # Data structures
```

**Key Components:**

```rust
// src/colony/state/backend.rs

pub struct GitBackedState {
    state_dir: PathBuf,        // .colony/state/
    cache_db: PathBuf,         // .colony/cache/state.db
    cache: SqliteCache,
    config: StateConfig,
    dirty: Arc<AtomicBool>,    // Track if needs export
}

impl GitBackedState {
    pub async fn new(repo_root: &Path) -> Result<Self> {
        let state_dir = repo_root.join(".colony/state");
        let cache_db = repo_root.join(".colony/cache/state.db");

        // Create directories
        tokio::fs::create_dir_all(&state_dir).await?;
        tokio::fs::create_dir_all(&cache_db.parent().unwrap()).await?;

        // Initialize cache
        let cache = SqliteCache::new(&cache_db).await?;

        // Import existing JSONL files
        let mut state = Self {
            state_dir,
            cache_db,
            cache,
            config: StateConfig::default(),
            dirty: Arc::new(AtomicBool::new(false)),
        };

        state.import_all().await?;

        Ok(state)
    }

    pub async fn write_task(&self, task: &Task) -> Result<()> {
        // 1. Write to cache (fast)
        self.cache.upsert_task(task).await?;

        // 2. Mark dirty for export
        self.dirty.store(true, Ordering::Relaxed);

        // 3. Schedule export (debounced)
        self.schedule_export().await?;

        Ok(())
    }

    pub async fn read_task(&self, id: &str) -> Result<Option<Task>> {
        // Check if import needed
        if self.needs_import("tasks.jsonl").await? {
            self.import_file("tasks.jsonl").await?;
        }

        // Read from cache
        self.cache.get_task(id).await
    }

    pub async fn query_ready_tasks(&self) -> Result<Vec<Task>> {
        self.cache.query_tasks(
            "status = 'ready' OR (
                SELECT COUNT(*)
                FROM json_each(blockers) AS blocker
                JOIN tasks ON tasks.id = blocker.value
                WHERE tasks.status != 'completed'
            ) = 0"
        ).await
    }
}
```

### Phase 2: Auto-Sync (Week 2)

**Export on Changes:**
```rust
impl GitBackedState {
    async fn schedule_export(&self) -> Result<()> {
        // Debounce: wait 5 seconds of inactivity
        tokio::time::sleep(Duration::from_secs(5)).await;

        if self.dirty.load(Ordering::Relaxed) {
            self.export_all().await?;
            self.dirty.store(false, Ordering::Relaxed);
        }

        Ok(())
    }

    async fn export_all(&self) -> Result<()> {
        // Export tasks
        let tasks = self.cache.get_all_tasks().await?;
        self.export_to_jsonl("tasks.jsonl", &tasks).await?;

        // Export workflows
        let workflows = self.cache.get_all_workflows().await?;
        self.export_to_jsonl("workflows.jsonl", &workflows).await?;

        // Auto-commit if configured
        if self.config.auto_commit {
            self.git_commit("Update colony state").await?;
        }

        Ok(())
    }

    async fn export_to_jsonl<T: Serialize>(
        &self,
        filename: &str,
        items: &[T]
    ) -> Result<()> {
        let path = self.state_dir.join(filename);
        let mut file = tokio::fs::File::create(&path).await?;

        for item in items {
            let json = serde_json::to_string(item)?;
            file.write_all(json.as_bytes()).await?;
            file.write_all(b"\n").await?;
        }

        file.sync_all().await?;
        Ok(())
    }
}
```

**Import on Read:**
```rust
impl GitBackedState {
    async fn needs_import(&self, filename: &str) -> Result<bool> {
        let file_path = self.state_dir.join(filename);

        if !file_path.exists() {
            return Ok(false);
        }

        // Check if file is newer than last import
        let metadata = tokio::fs::metadata(&file_path).await?;
        let modified = metadata.modified()?;

        let last_import = self.cache.get_last_import(filename).await?;

        Ok(last_import.is_none() || modified > last_import.unwrap())
    }

    async fn import_file(&self, filename: &str) -> Result<()> {
        let file_path = self.state_dir.join(filename);

        match filename {
            "tasks.jsonl" => self.import_tasks(&file_path).await?,
            "workflows.jsonl" => self.import_workflows(&file_path).await?,
            _ => return Err(Error::UnknownFile(filename.to_string())),
        }

        // Update last import timestamp
        self.cache.set_last_import(filename, SystemTime::now()).await?;

        Ok(())
    }

    async fn import_tasks(&self, path: &Path) -> Result<()> {
        let file = tokio::fs::File::open(path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();

        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }

            let task: Task = serde_json::from_str(&line)?;
            self.cache.upsert_task(&task).await?;
        }

        Ok(())
    }
}
```

### Phase 3: Git Integration (Week 3)

```rust
impl GitBackedState {
    async fn git_commit(&self, message: &str) -> Result<()> {
        // Add state files
        Command::new("git")
            .args(&["add", ".colony/state/"])
            .current_dir(&self.repo_root)
            .output()
            .await?;

        // Commit with [skip ci] to avoid triggering builds
        Command::new("git")
            .args(&["commit", "-m", &format!("{} [skip ci]", message)])
            .current_dir(&self.repo_root)
            .output()
            .await?;

        Ok(())
    }

    async fn git_pull(&self) -> Result<()> {
        Command::new("git")
            .args(&["pull", "origin", &self.config.branch])
            .current_dir(&self.repo_root)
            .output()
            .await?;

        // Re-import after pull
        self.import_all().await?;

        Ok(())
    }

    async fn git_push(&self) -> Result<()> {
        Command::new("git")
            .args(&["push", "origin", &self.config.branch])
            .current_dir(&self.repo_root)
            .output()
            .await?;

        Ok(())
    }
}
```

### Phase 4: CLI Integration (Week 3-4)

```rust
// src/colony/cli/task.rs

pub async fn run_task_command(args: TaskArgs) -> Result<()> {
    let state = GitBackedState::new(Path::new(".")).await?;

    match args.command {
        TaskCommand::Create { title, blockers } => {
            let task = Task::new(title, blockers);
            state.write_task(&task).await?;
            println!("Created task: {}", task.id);
        }

        TaskCommand::Ready => {
            let tasks = state.query_ready_tasks().await?;
            println!("Ready tasks ({}):", tasks.len());
            for task in tasks {
                println!("  {}: {}", task.id, task.title);
            }
        }

        TaskCommand::Claim { task_id } => {
            state.claim_task(&task_id, "current-agent").await?;
            println!("Claimed task: {}", task_id);
        }

        TaskCommand::Complete { task_id } => {
            state.complete_task(&task_id).await?;
            println!("Completed task: {}", task_id);
        }
    }

    Ok(())
}
```

**CLI Usage:**
```bash
# Create task
colony task create "Add user auth"
colony task create "Write tests" --blocks task-a1b2c3

# Query ready work
colony task ready

# Claim and complete
colony task claim task-a1b2c3
colony task complete task-a1b2c3

# View all tasks
colony task list
colony task show task-a1b2c3
```

## Configuration in colony.yml

```yaml
# colony.yml
shared_state:
  backend: git-backed

  # Storage location
  location: in-repo  # or: external
  path: .colony/state/
  cache: .colony/cache/state.db

  # Git settings
  branch: main  # or: colony-state for protected branches
  auto_commit: true
  auto_push: false  # Manual push gives user control
  commit_message: "Update colony state [skip ci]"

  # Sync settings
  auto_pull: false  # Manual pull gives user control
  sync_on_start: true  # Pull when colony starts

  # Export settings
  debounce_ms: 5000  # Wait 5s of inactivity before export

  # Schemas
  schemas:
    - name: tasks
      file: tasks.jsonl
      cache: true

    - name: workflows
      file: workflows.jsonl
      cache: true

    - name: agent_memory
      file: memory/{agent_id}.jsonl
      cache: false  # Too dynamic
```

## .gitignore Updates

```gitignore
# Colony cache (never commit)
.colony/cache/

# Colony projects (agent working directories)
.colony/projects/

# Colony worktrees
.colony/worktrees/

# Keep state (this is the source of truth)
# .colony/state/ is TRACKED
```

## Protected Branch Support

For repos with protected main branch:

```yaml
shared_state:
  backend: git-backed
  location: in-repo

  # Use separate branch for state
  branch: colony-state
  auto_commit: true
  auto_push: true

  # Sync strategy
  sync_to_main: on_pr_merge  # Merge state when PRs merge
```

**Workflow:**
1. Work happens on feature branch with `colony-state` branch for state
2. State commits go to `colony-state`
3. When PR merges to main, state is merged too
4. All agents pull `colony-state` for latest state

## Multi-Machine Coordination

**Machine A:**
```bash
# Create task
colony task create "Add auth"
# Exports to .colony/state/tasks.jsonl
# Auto-commits to local git

# Push when ready to share
git push origin colony-state
```

**Machine B:**
```bash
# Pull latest state
git pull origin colony-state
# Auto-imports on next command

# Query ready work
colony task ready
# Sees task created on Machine A!

# Claim and work on it
colony task claim task-a1b2c3
```

## Migration Path

### From Current Colony

1. **Initialize state:**
   ```bash
   colony state init
   # Creates .colony/state/
   # Migrates current messages to JSONL
   ```

2. **Backward compatibility:**
   - Old messaging still works
   - Gradually migrate to task-based workflow
   - Both can coexist during transition

### From Beads

If project uses Beads:
```bash
colony state import --from-beads
# Converts .beads/issues.jsonl to .colony/state/tasks.jsonl
# Maps Beads concepts to Colony
```

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_write_and_read_task() {
        let temp_dir = TempDir::new().unwrap();
        let state = GitBackedState::new(temp_dir.path()).await.unwrap();

        let task = Task::new("Test task");
        state.write_task(&task).await.unwrap();

        let read = state.read_task(&task.id).await.unwrap();
        assert_eq!(read.unwrap().title, "Test task");
    }

    #[tokio::test]
    async fn test_export_import_cycle() {
        let temp_dir = TempDir::new().unwrap();
        let state = GitBackedState::new(temp_dir.path()).await.unwrap();

        let task = Task::new("Test");
        state.write_task(&task).await.unwrap();
        state.export_all().await.unwrap();

        // Clear cache
        state.cache.clear().await.unwrap();

        // Re-import
        state.import_all().await.unwrap();

        let read = state.read_task(&task.id).await.unwrap();
        assert!(read.is_some());
    }
}
```

## Performance Targets

- **Write operation**: <10ms (cache write)
- **Read operation**: <5ms (cache read)
- **Export to JSONL**: <100ms (for 1000 tasks)
- **Import from JSONL**: <200ms (for 1000 tasks)
- **Git commit**: <500ms
- **Cache rebuild**: <1s (for 10,000 tasks)

## Future Enhancements

1. **Compaction**: Periodically compact JSONL files (remove old/deleted entries)
2. **Encryption**: Encrypt sensitive state data
3. **Conflict resolution**: Smart merging when multiple agents modify same task
4. **State branches**: Different states for different git branches
5. **Remote state**: Option to push state to separate remote repo

---

## Next Steps

1. Implement Phase 1 (core infrastructure)
2. Add basic CLI commands
3. Test with simple workflows
4. Gather feedback
5. Iterate and improve

This approach gives us Beads-like benefits while keeping Colony's flexibility!
