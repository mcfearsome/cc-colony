# Beads Integration: Git-Backed State for Colony

## Overview

[Beads](https://github.com/steveyegge/beads) is a lightweight, git-backed issue tracking system designed specifically for AI agents. Its architecture and design patterns align perfectly with several Colony roadmap features, particularly around shared state management and task orchestration.

**Key Insight:** Beads demonstrates that git itself can be a distributed, persistent state layer for multi-agent systems—no external database required.

---

## What We Can Learn from Beads

### 1. Git-Backed State Storage

**Beads' Approach:**
- Source of truth: `.beads/issues.jsonl` (git-tracked)
- Local cache: SQLite for fast queries
- Auto-sync: 5-second debounce exports SQLite → JSONL
- Bidirectional: Import JSONL → SQLite on startup/pull

**Benefits:**
- ✅ No external database server needed
- ✅ Version control for state changes
- ✅ Works with existing git workflows
- ✅ Distributed by default
- ✅ Offline-capable with eventual sync
- ✅ Audit trail built-in

**Apply to Colony:**
```yaml
# colony.yml
shared_state:
  backend: git-backed  # New backend type!
  storage:
    format: jsonl  # or parquet, sqlite, etc.
    path: .colony/state/
    cache: sqlite  # Local cache for queries

  schemas:
    - name: task_queue
      type: queue
      file: .colony/state/tasks.jsonl

    - name: agent_memory
      type: key-value
      file: .colony/state/memory.jsonl
      ttl: 7d

    - name: workflow_state
      type: document
      file: .colony/state/workflows.jsonl
```

**Implementation:**
```rust
// src/colony/state/git_backend.rs

pub struct GitBackedState {
    git_path: PathBuf,      // .colony/state/
    cache: SqliteCache,     // Local SQLite
    auto_sync: bool,        // Auto-commit changes
    debounce_ms: u64,       // Wait before committing
}

impl GitBackedState {
    pub async fn write(&self, key: &str, value: &Value) -> Result<()> {
        // 1. Write to SQLite cache (fast)
        self.cache.insert(key, value).await?;

        // 2. Schedule export to JSONL (debounced)
        self.schedule_export().await?;

        Ok(())
    }

    pub async fn read(&self, key: &str) -> Result<Option<Value>> {
        // 1. Check if JSONL is newer than cache
        if self.needs_import().await? {
            self.import_from_jsonl().await?;
        }

        // 2. Read from SQLite cache (fast)
        self.cache.get(key).await
    }

    async fn export_to_jsonl(&self) -> Result<()> {
        // Export SQLite to JSONL files
        for schema in &self.schemas {
            let rows = self.cache.query_all(&schema.name).await?;
            let jsonl_path = self.git_path.join(&schema.file);

            // Write as JSONL
            for row in rows {
                writeln!(file, "{}", serde_json::to_string(&row)?)?;
            }
        }

        // Auto-commit if enabled
        if self.auto_sync {
            self.git_commit("Update colony state").await?;
        }

        Ok(())
    }
}
```

---

### 2. Collision-Free ID System

**Beads' Approach:**
- Hash-based IDs: `bd-a1b2`, `bd-f14c` (4-6 char hex)
- No coordination required between agents
- Progressive length scaling as database grows
- Hierarchical children: `bd-a3f8.1`, `bd-a3f8.2`

**Why It Matters:**
Multiple agents creating work items simultaneously across different machines would collide with sequential IDs. Hash-based IDs eliminate this entirely.

**Apply to Colony:**

```rust
// src/colony/id.rs

use sha2::{Sha256, Digest};
use std::time::SystemTime;

pub struct ColonyIdGenerator {
    prefix: String,      // "task", "workflow", "message"
    length: usize,       // ID length (4-8 chars)
    existing: HashSet<String>,
}

impl ColonyIdGenerator {
    pub fn generate(&mut self) -> String {
        loop {
            // Create hash from timestamp + random + counter
            let mut hasher = Sha256::new();
            hasher.update(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().to_le_bytes());
            hasher.update(rand::random::<u64>().to_le_bytes());
            let hash = hasher.finalize();

            // Take first N hex characters
            let id = format!("{}-{}", self.prefix, hex::encode(&hash[..self.length/2]));

            // Check for collision (extremely unlikely)
            if !self.existing.contains(&id) {
                self.existing.insert(id.clone());
                return id;
            }
        }
    }

    pub fn child_id(&self, parent_id: &str, index: usize) -> String {
        format!("{}.{}", parent_id, index)
    }
}
```

**Usage:**
```rust
// Creating tasks with collision-free IDs
let mut gen = ColonyIdGenerator::new("task", 6);

// Different agents can create these simultaneously
let task1 = gen.generate(); // "task-a1b2c3"
let task2 = gen.generate(); // "task-f4e5d6"
let subtask = gen.child_id(&task1, 1); // "task-a1b2c3.1"
```

**Configuration:**
```yaml
# colony.yml
id_generation:
  strategy: hash-based  # or sequential, uuid
  prefix_by_type:
    task: task
    workflow: wf
    message: msg
  length: 6  # Hex characters (2^24 = 16M combinations)
  scale_at: 10000  # Increase length after N items
```

---

### 3. Dependency & Ready-Work System

**Beads' Approach:**
- Four relationship types: blockers, related, parent-child, discovered-by
- "Ready work" query: issues with no open blockers
- Agents autonomously select what to work on

**Apply to Colony:**

```jsonl
# .colony/state/tasks.jsonl

{"id":"task-a1b2","title":"Add user auth","status":"ready","assigned":null,"blockers":[]}
{"id":"task-c3d4","title":"Write auth tests","status":"blocked","assigned":null,"blockers":["task-a1b2"]}
{"id":"task-e5f6","title":"Deploy to prod","status":"blocked","assigned":null,"blockers":["task-a1b2","task-c3d4"]}
{"id":"task-g7h8","title":"Update docs","status":"ready","assigned":null,"blockers":[]}
```

**Agent Queries:**
```bash
# Agent asks: what can I work on?
./colony_task.sh ready

# Output:
# task-a1b2: Add user auth (no blockers)
# task-g7h8: Update docs (no blockers)

# Agent claims a task
./colony_task.sh claim task-a1b2

# Agent marks complete (unblocks dependent tasks)
./colony_task.sh complete task-a1b2

# Now task-c3d4 becomes ready automatically
```

**Implementation:**
```rust
// src/colony/tasks.rs

pub struct TaskQueue {
    state: GitBackedState,
}

impl TaskQueue {
    pub async fn get_ready_tasks(&self) -> Result<Vec<Task>> {
        // Query: tasks with status=ready or blocked=[]
        self.state.query(
            "SELECT * FROM tasks WHERE status = 'ready' OR (
                SELECT COUNT(*) FROM tasks AS blockers
                WHERE blockers.id IN (SELECT value FROM json_each(tasks.blockers))
                AND blockers.status != 'completed'
            ) = 0"
        ).await
    }

    pub async fn complete_task(&self, task_id: &str) -> Result<()> {
        // 1. Mark task as completed
        self.state.update("tasks", task_id, |task| {
            task.status = Status::Completed;
            task.completed_at = Some(Utc::now());
        }).await?;

        // 2. Find tasks blocked by this one
        let dependent_tasks = self.state.query(
            "SELECT * FROM tasks WHERE blockers LIKE ?",
            &[&format!("%{}%", task_id)]
        ).await?;

        // 3. Update their status if all blockers resolved
        for task in dependent_tasks {
            if self.all_blockers_resolved(&task).await? {
                self.state.update("tasks", &task.id, |t| {
                    t.status = Status::Ready;
                }).await?;
            }
        }

        Ok(())
    }
}
```

**Configuration:**
```yaml
# colony.yml
task_management:
  backend: git-backed
  auto_unblock: true  # Automatically mark tasks ready when blockers resolve
  claim_timeout: 1h   # Release claimed tasks if not completed
  notification: true  # Notify when tasks become ready

  relationships:
    - type: blocks
      description: "Task must complete before dependent can start"
    - type: related
      description: "Tasks are connected but not blocking"
    - type: discovered-by
      description: "Task found while working on another"
```

---

### 4. Multi-Agent Coordination

**Beads' Approach:**
- Git as coordination layer (no central server)
- Each agent pulls, works locally, pushes
- Merge conflicts resolved automatically via JSONL append-only logs
- Protected branch compatible (uses sync branch)

**Apply to Colony:**

```yaml
# colony.yml - Multi-machine colony
federation:
  mode: git-sync  # New mode!
  sync_branch: colony-state  # Dedicated branch for state
  auto_pull: 5m   # Pull remote changes every 5 minutes
  auto_push: true # Push local changes on export

  conflict_resolution: last-write-wins  # or: merge, manual
```

**Workflow:**
```bash
# Agent 1 (Machine A):
colony start
# Creates task-a1b2 locally
# Exports to .colony/state/tasks.jsonl
# Commits and pushes to colony-state branch

# Agent 2 (Machine B):
colony start
# Pulls colony-state branch
# Imports .colony/state/tasks.jsonl to SQLite
# Sees task-a1b2 in queue
# Creates task-c3d4 (no collision!)
# Exports, commits, pushes

# Agent 1:
# Auto-pulls after 5 minutes
# Imports updated tasks.jsonl
# Now sees task-c3d4
```

**Protected Branch Workaround:**
```yaml
# For repos with protected main branch
federation:
  mode: git-sync
  main_branch: main
  sync_branch: colony-state  # Unprotected branch for state
  sync_on_pr_merge: true     # Sync state when PRs merge
```

**Implementation:**
```rust
// src/colony/federation/git_sync.rs

pub struct GitSyncFederation {
    sync_branch: String,
    auto_pull_interval: Duration,
    auto_push: bool,
}

impl GitSyncFederation {
    pub async fn start_sync_loop(&self) -> Result<()> {
        loop {
            tokio::time::sleep(self.auto_pull_interval).await;

            // Pull remote state
            self.git_pull().await?;

            // Import to local cache
            self.import_state().await?;

            // Export local changes
            self.export_state().await?;

            // Push to remote (if auto_push)
            if self.auto_push {
                self.git_push().await?;
            }
        }
    }

    async fn git_pull(&self) -> Result<()> {
        Command::new("git")
            .args(&["pull", "origin", &self.sync_branch])
            .output()
            .await?;
        Ok(())
    }

    async fn git_push(&self) -> Result<()> {
        // Commit changes
        Command::new("git")
            .args(&["add", ".colony/state/"])
            .output()
            .await?;

        Command::new("git")
            .args(&["commit", "-m", "Update colony state [skip ci]"])
            .output()
            .await?;

        // Push to sync branch
        Command::new("git")
            .args(&["push", "origin", &self.sync_branch])
            .output()
            .await?;

        Ok(())
    }
}
```

---

### 5. Session Handoff Protocol

**Beads' Approach:**
- Explicit session-end responsibilities for agents
- Ensures work is filed, tested, synced before handoff
- Prevents "amnesia" between sessions

**Apply to Colony:**

```yaml
# colony.yml
session:
  on_end:
    - name: file_work
      description: "Create tasks for any TODOs or unfinished work"
      agent: coordinator
      required: true

    - name: sync_state
      description: "Export and push state to git"
      agent: system
      required: true

    - name: run_tests
      description: "Ensure tests pass before ending session"
      agent: test-runner
      required: false

    - name: update_status
      description: "Update task statuses and unblock dependents"
      agent: coordinator
      required: true

  on_start:
    - name: import_state
      description: "Pull latest state from git"
      agent: system
      required: true

    - name: show_ready_work
      description: "Display tasks ready to work on"
      agent: coordinator
      required: true

    - name: review_recent_changes
      description: "Show what changed since last session"
      agent: coordinator
      required: false
```

**Implementation:**
```rust
// src/colony/session.rs

pub struct SessionManager {
    config: SessionConfig,
    state: GitBackedState,
}

impl SessionManager {
    pub async fn end_session(&self) -> Result<()> {
        println!("\n=== Ending Colony Session ===\n");

        for hook in &self.config.on_end {
            if hook.required {
                print!("⚠ Required: {} ... ", hook.description);
            } else {
                print!("○ Optional: {} ... ", hook.description);
            }

            match self.execute_hook(hook).await {
                Ok(_) => println!("✓"),
                Err(e) if hook.required => {
                    println!("✗ FAILED: {}", e);
                    return Err(e);
                }
                Err(e) => println!("⚠ Skipped: {}", e),
            }
        }

        println!("\n✓ Session ended successfully\n");
        Ok(())
    }

    pub async fn start_session(&self) -> Result<()> {
        println!("\n=== Starting Colony Session ===\n");

        for hook in &self.config.on_start {
            // Execute start hooks
            self.execute_hook(hook).await?;
        }

        // Show session summary
        self.show_session_summary().await?;

        Ok(())
    }

    async fn show_session_summary(&self) -> Result<()> {
        let ready_tasks = self.state.get_ready_tasks().await?;
        let recent_changes = self.state.get_changes_since_last_session().await?;

        println!("📋 Ready Work ({} tasks):", ready_tasks.len());
        for task in ready_tasks.iter().take(5) {
            println!("  - {}: {}", task.id, task.title);
        }

        println!("\n📝 Recent Changes ({}):", recent_changes.len());
        for change in recent_changes.iter().take(5) {
            println!("  - {} {} by {}", change.timestamp, change.description, change.agent);
        }

        println!();
        Ok(())
    }
}
```

**CLI Integration:**
```bash
# Graceful session end
colony stop

# Output:
# === Ending Colony Session ===
#
# ⚠ Required: Create tasks for unfinished work ... ✓
# ⚠ Required: Export and push state to git ... ✓
# ○ Optional: Run tests ... ✓
# ⚠ Required: Update task statuses ... ✓
#
# ✓ Session ended successfully

# Next session start
colony start

# Output:
# === Starting Colony Session ===
#
# Importing latest state from git ... ✓
# Updating task queue ... ✓
#
# 📋 Ready Work (3 tasks):
#   - task-a1b2: Add user authentication
#   - task-g7h8: Update documentation
#   - task-k9l0: Fix bug in API
#
# 📝 Recent Changes (2):
#   - 2024-11-17 14:30 task-c3d4 completed by backend-agent
#   - 2024-11-17 14:25 task-e5f6 created by coordinator
```

---

## Integration Roadmap

### Phase 1: Git-Backed State (Replaces External DB)

**What:** Implement git-backed state storage as Colony's primary state layer

**Benefits:**
- ✅ No Redis/Postgres dependency
- ✅ Works offline
- ✅ Version-controlled state
- ✅ Distributed by default

**Implementation:**
1. Create `GitBackedState` backend
2. JSONL export/import
3. SQLite caching layer
4. Auto-sync on state changes

**Timeline:** 2-3 weeks

---

### Phase 2: Task Management (Replaces Manual Workflows)

**What:** Beads-style task queue with dependencies and ready-work detection

**Benefits:**
- ✅ Agents autonomously select work
- ✅ Dependency management
- ✅ No lost work between sessions

**Implementation:**
1. Task schema in JSONL
2. Dependency tracking
3. Ready-work queries
4. CLI: `colony task create/claim/complete/ready`

**Timeline:** 1-2 weeks

---

### Phase 3: Collision-Free IDs (Enables Multi-Agent)

**What:** Hash-based ID generation for all colony entities

**Benefits:**
- ✅ Multiple agents create items safely
- ✅ No coordination overhead
- ✅ Hierarchical IDs

**Implementation:**
1. `ColonyIdGenerator` with SHA-256
2. Progressive length scaling
3. Update all entity creation

**Timeline:** 1 week

---

### Phase 4: Session Management (Prevents Amnesia)

**What:** Explicit session start/end hooks

**Benefits:**
- ✅ Clean handoffs between sessions
- ✅ No lost context
- ✅ Required checks before ending

**Implementation:**
1. Session hooks in `colony.yml`
2. `colony start/stop` integration
3. Summary display

**Timeline:** 1 week

---

### Phase 5: Git Federation (Multi-Machine)

**What:** Multiple machines sharing state via git sync

**Benefits:**
- ✅ True distributed colony
- ✅ No central server
- ✅ Offline-first

**Implementation:**
1. Sync branch strategy
2. Auto-pull/push loop
3. Conflict resolution
4. Protected branch workaround

**Timeline:** 2 weeks

---

## Comparison: Beads Integration vs. Original Roadmap

| Feature | Original Plan | Beads-Inspired Approach |
|---------|---------------|------------------------|
| **State Backend** | Redis/Postgres | Git + SQLite |
| **Setup Complexity** | External services | Zero setup |
| **Distribution** | Network-based | Git-native |
| **Offline Mode** | Not supported | Fully supported |
| **Version Control** | Application-level | Built-in (git) |
| **Multi-Agent IDs** | Sequential (conflicts) | Hash-based (collision-free) |
| **Task Queue** | Redis queue | JSONL + dependency graph |
| **Federation** | HTTP/RPC | Git sync |
| **State Queries** | Redis commands | SQL (SQLite) |
| **Durability** | DB snapshots | Git commits |

---

## Recommended Next Steps

1. **Prototype Git-Backed State** (1 week)
   - Implement basic JSONL + SQLite
   - Test with simple key-value operations
   - Validate git sync works

2. **Add Task Management** (1 week)
   - Task schema and CLI
   - Dependency tracking
   - Ready-work queries

3. **Integrate with Colony** (1 week)
   - Replace current message queue with git-backed
   - Update `colony start/stop`
   - Add session hooks

4. **Documentation & Examples** (3 days)
   - Migration guide from current approach
   - Example workflows
   - Best practices

5. **Community Feedback** (ongoing)
   - Share prototype
   - Gather use cases
   - Iterate on design

---

## Conclusion

Beads demonstrates that **git is a brilliant state layer for multi-agent systems**. By adopting its core patterns—git-backed storage, hash-based IDs, dependency tracking, and session protocols—Colony can achieve:

- **Simpler deployment**: No external databases
- **Better developer experience**: State is version-controlled code
- **True distribution**: Works across machines via git
- **Offline resilience**: Agents work without network
- **Audit trails**: Every state change is a commit

This isn't just an implementation detail—it fundamentally changes what Colony can be:

**From**: Multi-agent platform requiring infrastructure
**To**: Multi-agent platform that's just git + code

The beauty is that developers already understand git. State management becomes familiar instead of foreign.

---

*Beads: https://github.com/steveyegge/beads*
*Colony Roadmap: docs/ROADMAP.md*
