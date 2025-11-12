/// Skills module - Handles installation of Claude Code skills
use crate::error::ColonyResult;
use std::fs;
use std::path::PathBuf;

// Embed the colony-message skill at compile time
const COLONY_MESSAGE_MD: &str = include_str!("../../.claude/skills/colony-message.md");

/// Get the user's home directory
fn get_home_dir() -> ColonyResult<PathBuf> {
    dirs::home_dir().ok_or_else(|| {
        crate::error::ColonyError::Colony(
            "Could not determine home directory".to_string()
        )
    })
}

/// Install the colony-message skill to ~/.claude/skills/
/// This makes it available system-wide for all Claude Code sessions
pub fn install_colony_message_skill() -> ColonyResult<()> {
    let home = get_home_dir()?;
    let skills_dir = home.join(".claude").join("skills");

    // Create the skills directory if it doesn't exist (including parent .claude dir)
    fs::create_dir_all(&skills_dir)?;

    // Write the colony-message skill file
    let skill_path = skills_dir.join("colony-message.md");
    fs::write(&skill_path, COLONY_MESSAGE_MD)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colony_message_skill_is_embedded() {
        // Verify the skill content is embedded and non-empty
        assert!(!COLONY_MESSAGE_MD.is_empty());
        assert!(COLONY_MESSAGE_MD.contains("Colony Message Skill"));
        assert!(COLONY_MESSAGE_MD.contains("colony_message.sh"));
    }
}
