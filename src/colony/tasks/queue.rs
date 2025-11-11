use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use super::{load_task, save_task, Task, TaskStatus};
use crate::error::ColonyResult;

/// Task queue manager
pub struct TaskQueue {
    colony_root: PathBuf,
}

impl TaskQueue {
    /// Create a new task queue
    pub fn new(colony_root: &Path) -> Self {
        Self {
            colony_root: colony_root.to_path_buf(),
        }
    }

    /// Initialize task queue directory structure
    pub fn initialize(&self) -> ColonyResult<()> {
        let tasks_dir = self.colony_root.join("tasks");
        fs::create_dir_all(tasks_dir.join("pending"))?;
        fs::create_dir_all(tasks_dir.join("claimed"))?;
        fs::create_dir_all(tasks_dir.join("in_progress"))?;
        fs::create_dir_all(tasks_dir.join("blocked"))?;
        fs::create_dir_all(tasks_dir.join("completed"))?;
        fs::create_dir_all(tasks_dir.join("cancelled"))?;
        Ok(())
    }

    /// Get directory for a task status
    fn status_dir(&self, status: &TaskStatus) -> PathBuf {
        let tasks_dir = self.colony_root.join("tasks");
        match status {
            TaskStatus::Pending => tasks_dir.join("pending"),
            TaskStatus::Claimed => tasks_dir.join("claimed"),
            TaskStatus::InProgress => tasks_dir.join("in_progress"),
            TaskStatus::Blocked => tasks_dir.join("blocked"),
            TaskStatus::Completed => tasks_dir.join("completed"),
            TaskStatus::Cancelled => tasks_dir.join("cancelled"),
        }
    }

    /// Get path to a task file
    fn task_path(&self, status: &TaskStatus, task_id: &str) -> PathBuf {
        self.status_dir(status).join(format!("{}.json", task_id))
    }

    /// Create a new task
    pub fn create_task(&self, task: Task) -> ColonyResult<()> {
        let path = self.task_path(&task.status, &task.id);
        save_task(&task, &path)?;
        Ok(())
    }

    /// Load a specific task
    pub fn load_task(&self, task_id: &str) -> ColonyResult<Option<Task>> {
        // Search all status directories
        for status in &[
            TaskStatus::Pending,
            TaskStatus::Claimed,
            TaskStatus::InProgress,
            TaskStatus::Blocked,
            TaskStatus::Completed,
            TaskStatus::Cancelled,
        ] {
            let path = self.task_path(status, task_id);
            if path.exists() {
                return Ok(Some(load_task(&path)?));
            }
        }
        Ok(None)
    }

    /// Load all tasks
    pub fn load_all_tasks(&self) -> ColonyResult<Vec<Task>> {
        let mut tasks = Vec::new();

        for status in &[
            TaskStatus::Pending,
            TaskStatus::Claimed,
            TaskStatus::InProgress,
            TaskStatus::Blocked,
            TaskStatus::Completed,
            TaskStatus::Cancelled,
        ] {
            let dir = self.status_dir(status);
            if !dir.exists() {
                continue;
            }

            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(task) = load_task(&path) {
                        tasks.push(task);
                    }
                }
            }
        }

        // Sort by priority (high to low), then by created_at
        tasks.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.created_at.cmp(&b.created_at))
        });

        Ok(tasks)
    }

    /// Load tasks with a specific status
    pub fn load_tasks_by_status(&self, status: &TaskStatus) -> ColonyResult<Vec<Task>> {
        let mut tasks = Vec::new();
        let dir = self.status_dir(status);

        if !dir.exists() {
            return Ok(tasks);
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(task) = load_task(&path) {
                    tasks.push(task);
                }
            }
        }

        tasks.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.created_at.cmp(&b.created_at))
        });

        Ok(tasks)
    }

    /// Load tasks assigned to an agent
    pub fn load_tasks_for_agent(&self, agent_id: &str) -> ColonyResult<Vec<Task>> {
        let all_tasks = self.load_all_tasks()?;
        Ok(all_tasks
            .into_iter()
            .filter(|t| t.is_assigned_to(agent_id))
            .collect())
    }

    /// Get set of completed task IDs
    pub fn get_completed_task_ids(&self) -> ColonyResult<HashSet<String>> {
        let tasks = self.load_tasks_by_status(&TaskStatus::Completed)?;
        Ok(tasks.into_iter().map(|t| t.id).collect())
    }

    /// Find available tasks for an agent to claim
    pub fn find_claimable_tasks(&self, agent_id: &str) -> ColonyResult<Vec<Task>> {
        let pending = self.load_tasks_by_status(&TaskStatus::Pending)?;
        let completed_ids = self.get_completed_task_ids()?;

        Ok(pending
            .into_iter()
            .filter(|t| t.can_claim(agent_id, &completed_ids))
            .collect())
    }

    /// Update a task (moves it between directories if status changed)
    pub fn update_task(&self, task: &Task) -> ColonyResult<()> {
        // Remove from all possible old locations
        for status in &[
            TaskStatus::Pending,
            TaskStatus::Claimed,
            TaskStatus::InProgress,
            TaskStatus::Blocked,
            TaskStatus::Completed,
            TaskStatus::Cancelled,
        ] {
            let old_path = self.task_path(status, &task.id);
            if old_path.exists() && status != &task.status {
                fs::remove_file(old_path)?;
            }
        }

        // Save to new location
        let new_path = self.task_path(&task.status, &task.id);
        save_task(task, &new_path)?;

        Ok(())
    }

    /// Delete a task
    pub fn delete_task(&self, task_id: &str) -> ColonyResult<bool> {
        let mut deleted = false;

        for status in &[
            TaskStatus::Pending,
            TaskStatus::Claimed,
            TaskStatus::InProgress,
            TaskStatus::Blocked,
            TaskStatus::Completed,
            TaskStatus::Cancelled,
        ] {
            let path = self.task_path(status, task_id);
            if path.exists() {
                fs::remove_file(path)?;
                deleted = true;
            }
        }

        Ok(deleted)
    }

    /// Get task statistics
    pub fn get_statistics(&self) -> ColonyResult<TaskStatistics> {
        let tasks = self.load_all_tasks()?;

        let mut stats = TaskStatistics {
            total: tasks.len(),
            ..Default::default()
        };

        for task in tasks {
            match task.status {
                TaskStatus::Pending => stats.pending += 1,
                TaskStatus::Claimed => stats.claimed += 1,
                TaskStatus::InProgress => stats.in_progress += 1,
                TaskStatus::Blocked => stats.blocked += 1,
                TaskStatus::Completed => stats.completed += 1,
                TaskStatus::Cancelled => stats.cancelled += 1,
            }
        }

        Ok(stats)
    }

    /// Get task assignments per agent
    pub fn get_agent_assignments(&self) -> ColonyResult<HashMap<String, Vec<Task>>> {
        let tasks = self.load_all_tasks()?;
        let mut assignments: HashMap<String, Vec<Task>> = HashMap::new();

        for task in tasks {
            if let Some(ref agent_id) = task.claimed_by {
                assignments
                    .entry(agent_id.clone())
                    .or_default()
                    .push(task.clone());
            } else if let Some(ref agent_id) = task.assigned_to {
                if agent_id != "auto" {
                    assignments
                        .entry(agent_id.clone())
                        .or_default()
                        .push(task.clone());
                }
            }
        }

        Ok(assignments)
    }
}

/// Task statistics
#[derive(Debug, Default)]
pub struct TaskStatistics {
    pub total: usize,
    pub pending: usize,
    pub claimed: usize,
    pub in_progress: usize,
    pub blocked: usize,
    pub completed: usize,
    pub cancelled: usize,
}

impl TaskStatistics {
    pub fn completion_percentage(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.completed as f64 / self.total as f64) * 100.0
    }

    pub fn active_count(&self) -> usize {
        self.claimed + self.in_progress
    }
}
