use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::colony::agent::AgentState;
use crate::colony::worktree;
use crate::colony::{Agent, ColonyConfig};
use crate::error::ColonyResult;

/// Manages a colony of agents
pub struct ColonyController {
    /// Path to the colony root directory
    colony_root: PathBuf,
    /// Active agents
    agents: HashMap<String, Agent>,
    /// Colony configuration
    config: ColonyConfig,
}

impl ColonyController {
    /// Create a new colony controller
    pub fn new(config: ColonyConfig) -> ColonyResult<Self> {
        let colony_root = PathBuf::from(".colony");

        // Create colony directory if it doesn't exist
        fs::create_dir_all(&colony_root)?;
        fs::create_dir_all(colony_root.join("worktrees"))?;
        fs::create_dir_all(colony_root.join("projects"))?;
        fs::create_dir_all(colony_root.join("logs"))?;

        Ok(Self {
            colony_root,
            agents: HashMap::new(),
            config,
        })
    }

    /// Initialize agents from configuration
    pub fn initialize_agents(&mut self) -> ColonyResult<()> {
        for agent_config in &self.config.agents {
            let agent_id = &agent_config.id;

            // Determine working directory based on configuration
            let worktree_path = if let Some(custom_dir) = agent_config.working_directory() {
                // Agent uses a custom directory
                custom_dir
            } else {
                // Agent uses a Git worktree
                self.colony_root.join("worktrees").join(agent_id)
            };

            let project_path = self.colony_root.join("projects").join(agent_id);
            let log_path = self
                .colony_root
                .join("logs")
                .join(format!("{}.log", agent_id));

            // Create project directory
            fs::create_dir_all(&project_path)?;

            // Validate custom directory exists if specified
            if agent_config.uses_custom_directory() && !worktree_path.exists() {
                return Err(crate::error::ColonyError::Colony(format!(
                    "Custom directory '{}' for agent '{}' does not exist",
                    worktree_path.display(),
                    agent_id
                )));
            }

            // Create agent
            let agent = Agent::new(agent_config.clone(), worktree_path, project_path, log_path);

            self.agents.insert(agent_id.clone(), agent);
        }

        Ok(())
    }

    /// Create worktrees for all agents (skips agents with custom directories)
    pub fn create_worktrees(&self) -> ColonyResult<()> {
        for agent in self.agents.values() {
            // Skip if agent uses a custom directory
            if agent.config.uses_custom_directory() {
                continue;
            }

            worktree::create_worktree(agent.id(), &self.colony_root)?;
        }
        Ok(())
    }

    /// Get an agent by ID
    pub fn get_agent(&self, id: &str) -> Option<&Agent> {
        self.agents.get(id)
    }

    /// Get a mutable agent by ID
    pub fn get_agent_mut(&mut self, id: &str) -> Option<&mut Agent> {
        self.agents.get_mut(id)
    }

    /// Get all agents
    pub fn agents(&self) -> &HashMap<String, Agent> {
        &self.agents
    }

    /// Get mutable access to all agents
    pub fn agents_mut(&mut self) -> &mut HashMap<String, Agent> {
        &mut self.agents
    }

    /// Save agent states to disk
    pub fn save_state(&self) -> ColonyResult<()> {
        let state_path = self.colony_root.join("state.json");

        let states: Vec<AgentState> = self.agents.values().map(AgentState::from).collect();

        let json = serde_json::to_string_pretty(&states)?;
        fs::write(state_path, json)?;

        Ok(())
    }

    /// Load agent states from disk
    pub fn load_state(&mut self) -> ColonyResult<()> {
        let state_path = self.colony_root.join("state.json");

        if !state_path.exists() {
            return Ok(());
        }

        let json = fs::read_to_string(state_path)?;
        let states: Vec<AgentState> = serde_json::from_str(&json)?;

        for state in states {
            if let Some(agent) = self.agents.get_mut(&state.id) {
                agent.status = state.status;
                agent.pid = state.pid;
            }
        }

        Ok(())
    }

    /// Remove all worktrees
    pub fn cleanup_worktrees(&self) -> ColonyResult<()> {
        for agent in self.agents.values() {
            // Skip agents with custom directories - they manage their own paths
            if agent.config.uses_custom_directory() {
                continue;
            }
            worktree::remove_worktree(&agent.worktree_path)?;
        }
        Ok(())
    }

    /// Get the colony root directory
    pub fn colony_root(&self) -> &Path {
        &self.colony_root
    }
}
