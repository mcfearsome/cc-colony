//! Git-backed state management for Colony
//!
//! This module implements Beads-inspired git-backed state storage, where state
//! is stored in JSONL files (git-tracked) with a SQLite cache for fast queries.

mod backend;
mod cache;
mod jsonl;
mod state_config;
mod types;

pub use backend::GitBackedState;
pub use cache::StateCache;
pub use state_config::{SharedStateConfig, StateBackend, StateLocation, StateSchema};
pub use types::{
    MemoryEntry, MemoryType, StepStatus, Task, TaskIdGenerator, TaskStatus, Workflow,
    WorkflowStatus, WorkflowStep,
};

// Re-export for convenience
pub use jsonl::{append_jsonl, read_jsonl, write_jsonl};
