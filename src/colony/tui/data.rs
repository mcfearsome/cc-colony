use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::colony::agent::AgentStatus;
use crate::colony::messaging::Message;
use crate::colony::tasks::{Task, TaskStatus};
use crate::colony::ColonyConfig;

/// Represents the current state of the colony
#[derive(Debug, Clone)]
pub struct ColonyData {
    pub agents: Vec<AgentInfo>,
    pub tasks: HashMap<TaskStatus, Vec<Task>>,
    pub messages: Vec<Message>,
    pub colony_root: PathBuf,
}

/// Information about an agent
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub id: String,
    pub role: String,
    pub focus: String,
    pub model: String,
    pub status: AgentStatus,
    pub pid: Option<u32>,
    pub current_task: Option<String>,
}

impl ColonyData {
    /// Load colony data from disk
    pub fn load(config_path: &Path) -> Result<Self, String> {
        let colony_root = PathBuf::from(".colony");

        if !colony_root.exists() {
            return Err("Colony not initialized. Run 'colony start' first.".to_string());
        }

        // Load configuration
        let config = ColonyConfig::load(config_path)
            .map_err(|e| format!("Failed to load colony config: {}", e))?;

        // Load agents
        let agents = Self::load_agents(&config, &colony_root)?;

        // Load tasks
        let tasks = Self::load_tasks(&colony_root)?;

        // Load messages
        let messages = Self::load_messages(&colony_root)?;

        Ok(Self {
            agents,
            tasks,
            messages,
            colony_root,
        })
    }

    fn load_agents(config: &ColonyConfig, colony_root: &Path) -> Result<Vec<AgentInfo>, String> {
        let mut agents = Vec::new();

        // Load state file if it exists
        let state_path = colony_root.join("state.json");
        let states: HashMap<String, (AgentStatus, Option<u32>)> = if state_path.exists() {
            let content = fs::read_to_string(&state_path)
                .map_err(|e| format!("Failed to read state file: {}", e))?;
            let states_vec: Vec<serde_json::Value> = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse state file: {}", e))?;

            states_vec
                .into_iter()
                .filter_map(|state| {
                    let id = state.get("id")?.as_str()?.to_string();
                    let status_str = state.get("status")?.as_str()?;
                    let status = match status_str {
                        "idle" => AgentStatus::Idle,
                        "running" => AgentStatus::Running,
                        "completed" => AgentStatus::Completed,
                        "failed" => AgentStatus::Failed,
                        _ => return None,
                    };
                    let pid = state.get("pid")?.as_u64().map(|p| p as u32);
                    Some((id, (status, pid)))
                })
                .collect()
        } else {
            HashMap::new()
        };

        // Build map of agent ID -> current task
        let task_assignments = Self::load_task_assignments(colony_root)?;

        for agent_config in &config.agents {
            let (status, pid) = states
                .get(&agent_config.id)
                .cloned()
                .unwrap_or((AgentStatus::Idle, None));

            let current_task = task_assignments.get(&agent_config.id).cloned();

            agents.push(AgentInfo {
                id: agent_config.id.clone(),
                role: agent_config.role.clone(),
                focus: agent_config.focus.clone(),
                model: agent_config.model.clone(),
                status,
                pid,
                current_task,
            });
        }

        Ok(agents)
    }

    fn load_task_assignments(colony_root: &Path) -> Result<HashMap<String, String>, String> {
        let mut assignments = HashMap::new();
        let in_progress_dir = colony_root.join("tasks/in_progress");

        if !in_progress_dir.exists() {
            return Ok(assignments);
        }

        if let Ok(entries) = fs::read_dir(&in_progress_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(task) = serde_json::from_str::<Task>(&content) {
                            if let Some(claimed_by) = &task.claimed_by {
                                assignments.insert(claimed_by.clone(), task.title.clone());
                            }
                        }
                    }
                }
            }
        }

        Ok(assignments)
    }

    fn load_tasks(colony_root: &Path) -> Result<HashMap<TaskStatus, Vec<Task>>, String> {
        let mut tasks_by_status = HashMap::new();
        let tasks_dir = colony_root.join("tasks");

        if !tasks_dir.exists() {
            return Ok(tasks_by_status);
        }

        let status_dirs = vec![
            (TaskStatus::Pending, "pending"),
            (TaskStatus::Claimed, "claimed"),
            (TaskStatus::InProgress, "in_progress"),
            (TaskStatus::Blocked, "blocked"),
            (TaskStatus::Completed, "completed"),
            (TaskStatus::Cancelled, "cancelled"),
        ];

        for (status, dir_name) in status_dirs {
            let dir_path = tasks_dir.join(dir_name);
            let mut status_tasks = Vec::new();

            if dir_path.exists() {
                if let Ok(entries) = fs::read_dir(&dir_path) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("json") {
                            if let Ok(content) = fs::read_to_string(&path) {
                                if let Ok(task) = serde_json::from_str::<Task>(&content) {
                                    status_tasks.push(task);
                                }
                            }
                        }
                    }
                }
            }

            // Sort tasks by priority and creation time
            status_tasks.sort_by(|a, b| {
                b.priority
                    .cmp(&a.priority)
                    .then_with(|| a.created_at.cmp(&b.created_at))
            });

            tasks_by_status.insert(status, status_tasks);
        }

        Ok(tasks_by_status)
    }

    fn load_messages(colony_root: &Path) -> Result<Vec<Message>, String> {
        let messages_dir = colony_root.join("messages");

        if !messages_dir.exists() {
            return Ok(Vec::new());
        }

        let mut messages = Vec::new();

        // Recursively walk all message directories
        fn walk_messages(dir: &Path, messages: &mut Vec<Message>) -> Result<(), String> {
            if !dir.exists() {
                return Ok(());
            }

            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();

                    if path.is_dir() {
                        walk_messages(&path, messages)?;
                    } else if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(message) = serde_json::from_str::<Message>(&content) {
                                messages.push(message);
                            }
                        }
                    }
                }
            }

            Ok(())
        }

        walk_messages(&messages_dir, &mut messages)?;

        // Sort by timestamp (most recent first) and deduplicate
        messages.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        messages.dedup_by(|a, b| a.id == b.id);

        // Keep only the most recent 100 messages
        messages.truncate(100);

        Ok(messages)
    }

    /// Get task counts by status
    pub fn task_counts(&self) -> HashMap<TaskStatus, usize> {
        self.tasks
            .iter()
            .map(|(status, tasks)| (status.clone(), tasks.len()))
            .collect()
    }

    /// Get total task count
    pub fn total_tasks(&self) -> usize {
        self.tasks.values().map(|tasks| tasks.len()).sum()
    }

    /// Get completion percentage
    pub fn completion_percentage(&self) -> u8 {
        let total = self.total_tasks();
        if total == 0 {
            return 0;
        }

        let completed = self
            .tasks
            .get(&TaskStatus::Completed)
            .map(|tasks| tasks.len())
            .unwrap_or(0);

        ((completed as f64 / total as f64) * 100.0) as u8
    }
}
