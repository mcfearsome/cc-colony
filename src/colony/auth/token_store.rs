use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::error::ColonyResult;
use super::oauth::OAuthToken;

pub struct TokenStore {
    path: PathBuf,
}

impl TokenStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Save token to disk
    pub fn save_token(&self, token: &OAuthToken) -> ColonyResult<()> {
        // Create directory if it doesn't exist
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Serialize token
        let json = serde_json::to_string_pretty(token)?;

        // Write to file
        fs::write(&self.path, json)?;

        // Set restrictive permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&self.path)?.permissions();
            perms.set_mode(0o600); // Only owner can read/write
            fs::set_permissions(&self.path, perms)?;
        }

        Ok(())
    }

    /// Load token from disk
    pub fn load_token(&self) -> ColonyResult<Option<OAuthToken>> {
        if !self.path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&self.path)?;
        let token: OAuthToken = serde_json::from_str(&contents)?;

        Ok(Some(token))
    }

    /// Delete stored token
    pub fn delete_token(&self) -> ColonyResult<()> {
        if self.path.exists() {
            fs::remove_file(&self.path)?;
        }
        Ok(())
    }
}
