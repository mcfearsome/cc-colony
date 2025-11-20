use super::oauth::OAuthToken;
use crate::error::ColonyResult;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub struct TokenStore {
    path: PathBuf,
    use_keyring: bool,
}

impl TokenStore {
    pub fn new(path: PathBuf) -> Self {
        // Try to use keyring if available, fallback to file storage
        let use_keyring = is_keyring_available();
        Self { path, use_keyring }
    }

    /// Save token to secure storage
    pub fn save_token(&self, token: &OAuthToken) -> ColonyResult<()> {
        if self.use_keyring {
            self.save_to_keyring(token)
        } else {
            self.save_to_file(token)
        }
    }

    /// Load token from secure storage
    pub fn load_token(&self) -> ColonyResult<Option<OAuthToken>> {
        if self.use_keyring {
            self.load_from_keyring()
        } else {
            self.load_from_file()
        }
    }

    /// Delete stored token
    pub fn delete_token(&self) -> ColonyResult<()> {
        if self.use_keyring {
            self.delete_from_keyring()
        } else {
            self.delete_from_file()
        }
    }

    // Keyring storage (secure, encrypted by OS)
    fn save_to_keyring(&self, token: &OAuthToken) -> ColonyResult<()> {
        let entry = keyring::Entry::new("colony-cli", "oauth-token")
            .map_err(|e| crate::error::ColonyError::Auth(format!("Keyring error: {}", e)))?;

        let json = serde_json::to_string(token)?;

        entry.set_password(&json).map_err(|e| {
            crate::error::ColonyError::Auth(format!("Failed to save to keyring: {}", e))
        })?;

        // Also save to file as backup (for cross-platform compatibility)
        let _ = self.save_to_file(token);

        Ok(())
    }

    fn load_from_keyring(&self) -> ColonyResult<Option<OAuthToken>> {
        let entry = keyring::Entry::new("colony-cli", "oauth-token")
            .map_err(|e| crate::error::ColonyError::Auth(format!("Keyring error: {}", e)))?;

        match entry.get_password() {
            Ok(json) => {
                let token: OAuthToken = serde_json::from_str(&json)?;
                Ok(Some(token))
            }
            Err(keyring::Error::NoEntry) => {
                // Try file fallback
                self.load_from_file()
            }
            Err(e) => {
                // On other errors, try file fallback
                eprintln!("Warning: Keyring error ({}), using file storage", e);
                self.load_from_file()
            }
        }
    }

    fn delete_from_keyring(&self) -> ColonyResult<()> {
        let entry = keyring::Entry::new("colony-cli", "oauth-token")
            .map_err(|e| crate::error::ColonyError::Auth(format!("Keyring error: {}", e)))?;

        let _ = entry.delete_password(); // Ignore errors if not found

        // Also delete file backup
        self.delete_from_file()?;

        Ok(())
    }

    // File storage (fallback, with restrictive permissions)
    fn save_to_file(&self, token: &OAuthToken) -> ColonyResult<()> {
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

    fn load_from_file(&self) -> ColonyResult<Option<OAuthToken>> {
        if !self.path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&self.path)?;
        let token: OAuthToken = serde_json::from_str(&contents)?;

        Ok(Some(token))
    }

    fn delete_from_file(&self) -> ColonyResult<()> {
        if self.path.exists() {
            fs::remove_file(&self.path)?;
        }
        Ok(())
    }
}

/// Check if keyring is available on this system
fn is_keyring_available() -> bool {
    // Try to create a test entry
    match keyring::Entry::new("colony-cli-test", "test") {
        Ok(entry) => {
            // Try to set and delete a test password
            let test_result = entry.set_password("test").is_ok();
            let _ = entry.delete_password(); // Clean up
            test_result
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_file_storage() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("tokens.json");
        let store = TokenStore {
            path,
            use_keyring: false,
        };

        // Create test token
        let token = OAuthToken {
            access_token: "test_access".to_string(),
            refresh_token: "test_refresh".to_string(),
            token_type: "Bearer".to_string(),
            expires_in: 3600,
            scope: "openid profile api".to_string(),
            expires_at: Some(1234567890),
        };

        // Save
        store.save_to_file(&token).unwrap();

        // Load
        let loaded = store.load_from_file().unwrap().unwrap();
        assert_eq!(loaded.access_token, token.access_token);
        assert_eq!(loaded.refresh_token, token.refresh_token);

        // Delete
        store.delete_from_file().unwrap();
        assert!(store.load_from_file().unwrap().is_none());
    }
}
