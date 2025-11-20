use std::fs;
use std::path::Path;

use crate::error::ColonyResult;

// Embedded agent skill files
const TMUX_PANE_TOOLS: &str = include_str!("../../.claude/skills/colony-agent-skills/tmux-pane-tools.md");
const NVIM_PANE_EDITING: &str = include_str!("../../.claude/skills/colony-agent-skills/nvim-pane-editing.md");
const OLLAMA_LOCAL_LLM: &str = include_str!("../../.claude/skills/colony-agent-skills/ollama-local-llm.md");
const BASH_SCRIPTING: &str = include_str!("../../.claude/skills/colony-agent-skills/bash-scripting.md");
const GIT_WORKFLOW: &str = include_str!("../../.claude/skills/colony-agent-skills/git-workflow.md");
const GH_CLI: &str = include_str!("../../.claude/skills/colony-agent-skills/gh-cli.md");
const CURL_API: &str = include_str!("../../.claude/skills/colony-agent-skills/curl-api.md");
const JQ_JSON: &str = include_str!("../../.claude/skills/colony-agent-skills/jq-json.md");

/// Install agent skills to both project and worktree directories
pub fn install_agent_skills(project_path: &Path, worktree_path: &Path) -> ColonyResult<()> {
    // Install to project directory
    let project_skills_dir = project_path.join(".claude/skills");
    fs::create_dir_all(&project_skills_dir)?;
    fs::write(project_skills_dir.join("tmux-pane-tools.md"), TMUX_PANE_TOOLS)?;
    fs::write(project_skills_dir.join("nvim-pane-editing.md"), NVIM_PANE_EDITING)?;
    fs::write(project_skills_dir.join("ollama-local-llm.md"), OLLAMA_LOCAL_LLM)?;
    fs::write(project_skills_dir.join("bash-scripting.md"), BASH_SCRIPTING)?;
    fs::write(project_skills_dir.join("git-workflow.md"), GIT_WORKFLOW)?;
    fs::write(project_skills_dir.join("gh-cli.md"), GH_CLI)?;
    fs::write(project_skills_dir.join("curl-api.md"), CURL_API)?;
    fs::write(project_skills_dir.join("jq-json.md"), JQ_JSON)?;

    // Also install to worktree directory (agents run from worktree)
    let worktree_skills_dir = worktree_path.join(".claude/skills");
    fs::create_dir_all(&worktree_skills_dir)
        .map_err(|e| crate::error::ColonyError::Colony(format!(
            "Failed to create worktree skills dir {}: {}",
            worktree_skills_dir.display(),
            e
        )))?;
    fs::write(worktree_skills_dir.join("tmux-pane-tools.md"), TMUX_PANE_TOOLS)
        .map_err(|e| crate::error::ColonyError::Colony(format!(
            "Failed to write tmux skill: {}",
            e
        )))?;
    fs::write(worktree_skills_dir.join("nvim-pane-editing.md"), NVIM_PANE_EDITING)
        .map_err(|e| crate::error::ColonyError::Colony(format!(
            "Failed to write nvim skill: {}",
            e
        )))?;
    fs::write(worktree_skills_dir.join("ollama-local-llm.md"), OLLAMA_LOCAL_LLM)
        .map_err(|e| crate::error::ColonyError::Colony(format!(
            "Failed to write ollama skill: {}",
            e
        )))?;
    fs::write(worktree_skills_dir.join("bash-scripting.md"), BASH_SCRIPTING)
        .map_err(|e| crate::error::ColonyError::Colony(format!(
            "Failed to write bash skill: {}",
            e
        )))?;
    fs::write(worktree_skills_dir.join("git-workflow.md"), GIT_WORKFLOW)
        .map_err(|e| crate::error::ColonyError::Colony(format!(
            "Failed to write git skill: {}",
            e
        )))?;
    fs::write(worktree_skills_dir.join("gh-cli.md"), GH_CLI)
        .map_err(|e| crate::error::ColonyError::Colony(format!(
            "Failed to write gh skill: {}",
            e
        )))?;
    fs::write(worktree_skills_dir.join("curl-api.md"), CURL_API)
        .map_err(|e| crate::error::ColonyError::Colony(format!(
            "Failed to write curl skill: {}",
            e
        )))?;
    fs::write(worktree_skills_dir.join("jq-json.md"), JQ_JSON)
        .map_err(|e| crate::error::ColonyError::Colony(format!(
            "Failed to write jq skill: {}",
            e
        )))?;

    Ok(())
}
