use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::ForgeResult;
use crate::swarm::agent::AgentState;
use crate::swarm::worktree;
use crate::swarm::{Agent, SwarmConfig};

/// Manages a swarm of agents
pub struct SwarmController {
    /// Path to the swarm root directory
    swarm_root: PathBuf,
    /// Active agents
    agents: HashMap<String, Agent>,
    /// Swarm configuration
    config: SwarmConfig,
}

impl SwarmController {
    /// Create a new swarm controller
    pub fn new(config: SwarmConfig) -> ForgeResult<Self> {
        let swarm_root = PathBuf::from(".forge-swarm");

        // Create swarm directory if it doesn't exist
        fs::create_dir_all(&swarm_root)?;
        fs::create_dir_all(swarm_root.join("worktrees"))?;
        fs::create_dir_all(swarm_root.join("projects"))?;
        fs::create_dir_all(swarm_root.join("logs"))?;

        Ok(Self {
            swarm_root,
            agents: HashMap::new(),
            config,
        })
    }

    /// Initialize agents from configuration
    pub fn initialize_agents(&mut self) -> ForgeResult<()> {
        for agent_config in &self.config.agents {
            let agent_id = &agent_config.id;

            // Determine working directory based on configuration
            let worktree_path = if let Some(custom_dir) = agent_config.working_directory() {
                // Agent uses a custom directory
                custom_dir
            } else {
                // Agent uses a Git worktree
                self.swarm_root.join("worktrees").join(agent_id)
            };

            let project_path = self.swarm_root.join("projects").join(agent_id);
            let log_path = self
                .swarm_root
                .join("logs")
                .join(format!("{}.log", agent_id));

            // Create project directory
            fs::create_dir_all(&project_path)?;

            // Validate custom directory exists if specified
            if agent_config.uses_custom_directory() && !worktree_path.exists() {
                return Err(crate::error::ForgeError::Swarm(format!(
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
    pub fn create_worktrees(&self) -> ForgeResult<()> {
        for agent in self.agents.values() {
            // Skip if agent uses a custom directory
            if agent.config.uses_custom_directory() {
                continue;
            }

            worktree::create_worktree(agent.id(), &self.swarm_root)?;
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
    pub fn save_state(&self) -> ForgeResult<()> {
        let state_path = self.swarm_root.join("state.json");

        let states: Vec<AgentState> = self.agents.values().map(AgentState::from).collect();

        let json = serde_json::to_string_pretty(&states)?;
        fs::write(state_path, json)?;

        Ok(())
    }

    /// Load agent states from disk
    pub fn load_state(&mut self) -> ForgeResult<()> {
        let state_path = self.swarm_root.join("state.json");

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
    pub fn cleanup_worktrees(&self) -> ForgeResult<()> {
        for agent in self.agents.values() {
            // Skip agents with custom directories - they manage their own paths
            if agent.config.uses_custom_directory() {
                continue;
            }
            worktree::remove_worktree(&agent.worktree_path)?;
        }
        Ok(())
    }

    /// Get the swarm root directory
    pub fn swarm_root(&self) -> &Path {
        &self.swarm_root
    }
}
