use std::path::{Path, PathBuf};
use std::io::Write;

use crate::colony::auth::{AuthProvider, oauth::OAuthFlow, token_store::TokenStore, providers};
use crate::colony::config::ColonyConfig;
use crate::error::ColonyResult;
use crate::utils;

/// Check if keyring is available
fn is_keyring_available() -> bool {
    match keyring::Entry::new("colony-cli-test", "test") {
        Ok(entry) => {
            let test_result = entry.set_password("test").is_ok();
            let _ = entry.delete_password();
            test_result
        }
        Err(_) => false,
    }
}

/// Get the default token path
fn get_token_path() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        home.join(".colony/auth/tokens.json")
    } else {
        PathBuf::from(".colony/auth/tokens.json")
    }
}

/// Login with OAuth
pub async fn login_oauth() -> ColonyResult<()> {
    println!("🔐 Colony Authentication - OAuth\n");
    println!("This will authenticate using your Claude.ai account");
    println!("(For Claude Pro/Max users)\n");

    let oauth_flow = OAuthFlow::new();
    let token = oauth_flow.authenticate().await?;

    // Save token
    let token_path = get_token_path();
    let token_store = TokenStore::new(token_path.clone());
    token_store.save_token(&token)?;

    utils::success("Authentication successful!");
    println!("\n✨ You can now use Colony with your Claude subscription.");

    // Check if keyring was used
    if is_keyring_available() {
        println!("🔒 Token securely stored in system keyring");
        println!("📄 Backup saved to: {}", token_path.display());
    } else {
        println!("💾 Token saved to: {}", token_path.display());
        println!("💡 Tip: Install keyring support for encrypted storage");
    }

    println!("\n📝 To use this authentication, update your colony.yml:");
    println!("   auth:");
    println!("     provider: anthropic-oauth");
    println!("     token_path: {}", token_path.display());

    Ok(())
}

/// Login with API key
pub async fn login_api_key(api_key: Option<String>) -> ColonyResult<()> {
    println!("🔐 Colony Authentication - API Key\n");

    let key = if let Some(k) = api_key {
        k
    } else {
        // Prompt for API key
        print!("Enter your Anthropic API key: ");
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        input.trim().to_string()
    };

    // Validate API key format
    if !key.starts_with("sk-ant-") {
        return Err(crate::error::ColonyError::Auth(
            "Invalid API key format. Should start with 'sk-ant-'".to_string()
        ));
    }

    // Test the API key
    println!("\n🔍 Validating API key...");
    providers::test_api_key(&key).await?;

    utils::success("API key is valid!");

    // Show configuration instructions
    println!("\n📝 Add this to your colony.yml:");
    println!("   auth:");
    println!("     provider: api-key");
    println!("     api_key: $ANTHROPIC_API_KEY");
    println!("\n💡 Or set environment variable:");
    println!("   export ANTHROPIC_API_KEY={}", key);
    println!("\n⚠️  Keep your API key secure! Don't commit it to version control.");

    Ok(())
}

/// Configure Bedrock authentication
pub async fn login_bedrock(
    region: Option<String>,
    profile: Option<String>,
) -> ColonyResult<()> {
    println!("🔐 Colony Authentication - AWS Bedrock\n");

    let region = region.unwrap_or_else(|| "us-east-1".to_string());
    let profile = profile.unwrap_or_else(|| "default".to_string());

    println!("Region: {}", region);
    println!("Profile: {}\n", profile);

    // Test AWS credentials
    println!("🔍 Testing AWS credentials...");
    providers::test_bedrock_access(&region, &profile).await?;

    utils::success("AWS credentials are valid!");

    println!("\n📝 Add this to your colony.yml:");
    println!("   auth:");
    println!("     provider: bedrock");
    println!("     region: {}", region);
    println!("     profile: {}", profile);

    Ok(())
}

/// Show authentication status
pub async fn status() -> ColonyResult<()> {
    println!("🔐 Authentication Status\n");

    // Try to load colony.yml
    let config_path = Path::new("colony.yml");
    if !config_path.exists() {
        println!("❌ No colony.yml found in current directory");
        println!("   Run 'colony init' to create a new colony");
        return Ok(());
    }

    let config = ColonyConfig::load(config_path)?;

    match &config.auth.provider {
        AuthProvider::ApiKey { api_key } => {
            println!("Provider: Anthropic API Key");

            if api_key.is_some() {
                println!("Status: ✅ Configured in colony.yml");
            } else if std::env::var("ANTHROPIC_API_KEY").is_ok() {
                println!("Status: ✅ Configured via environment variable");
            } else {
                println!("Status: ❌ Not configured");
                println!("\nRun: colony auth login --method api-key");
            }
        }

        AuthProvider::Bedrock { region, profile } => {
            println!("Provider: AWS Bedrock");
            println!("Region: {}", region);
            println!("Profile: {}", profile.as_ref().unwrap_or(&"default".to_string()));

            match providers::test_bedrock_access(region, profile.as_deref().unwrap_or("default")).await {
                Ok(_) => println!("Status: ✅ Connected"),
                Err(e) => println!("Status: ❌ Cannot connect - {}", e),
            }
        }

        AuthProvider::AnthropicOAuth { token_path } => {
            println!("Provider: Claude.ai OAuth (Pro/Max)");

            let token_store = TokenStore::new(PathBuf::from(token_path));
            if let Ok(Some(token)) = token_store.load_token() {
                if token.is_expired() {
                    println!("Status: ⚠️  Token expired");
                    println!("\nRun: colony auth refresh");
                } else {
                    println!("Status: ✅ Authenticated");

                    if is_keyring_available() {
                        println!("Storage: 🔒 System keyring (encrypted)");
                    } else {
                        println!("Storage: 📄 File-based");
                    }

                    if let Some(expires_at) = token.expires_at {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
                        let remaining = expires_at.saturating_sub(now);
                        let days = remaining / 86400;
                        let hours = (remaining % 86400) / 3600;

                        if days > 0 {
                            println!("Expires: in {} days, {} hours", days, hours);
                        } else {
                            println!("Expires: in {} hours", hours);
                        }
                    }
                }
            } else {
                println!("Status: ❌ Not authenticated");
                println!("\nRun: colony auth login");
            }
        }

        AuthProvider::VertexAI { project, location } => {
            println!("Provider: Google Cloud Vertex AI");
            println!("Project: {}", project);
            println!("Location: {}", location);
            println!("Status: ℹ️  Not yet implemented");
        }
    }

    Ok(())
}

/// Logout (remove credentials)
pub async fn logout() -> ColonyResult<()> {
    println!("🔐 Logging out...\n");

    let config_path = Path::new("colony.yml");
    if !config_path.exists() {
        println!("No colony.yml found");
        return Ok(());
    }

    let config = ColonyConfig::load(config_path)?;

    match &config.auth.provider {
        AuthProvider::AnthropicOAuth { token_path } => {
            let token_store = TokenStore::new(PathBuf::from(token_path));
            token_store.delete_token()?;
            utils::success("Logged out successfully");
            println!("\n🗑️  OAuth token deleted from: {}", token_path);
        }
        _ => {
            println!("💡 For API key or Bedrock authentication:");
            println!("   - Remove the auth section from colony.yml, or");
            println!("   - Unset environment variables (ANTHROPIC_API_KEY, AWS_*)");
        }
    }

    Ok(())
}

/// Refresh OAuth token
pub async fn refresh() -> ColonyResult<()> {
    println!("🔄 Refreshing authentication token...\n");

    let config_path = Path::new("colony.yml");
    if !config_path.exists() {
        return Err(crate::error::ColonyError::Auth(
            "No colony.yml found".to_string()
        ));
    }

    let config = ColonyConfig::load(config_path)?;

    match &config.auth.provider {
        AuthProvider::AnthropicOAuth { token_path } => {
            let token_store = TokenStore::new(PathBuf::from(token_path));
            let token = token_store.load_token()?
                .ok_or_else(|| crate::error::ColonyError::Auth(
                    "No token found. Run 'colony auth login' first".to_string()
                ))?;

            // Refresh token using OAuth flow
            let oauth_flow = OAuthFlow::new();
            let new_token = oauth_flow.refresh_token(&token.refresh_token).await?;

            // Save new token
            token_store.save_token(&new_token)?;

            utils::success("Token refreshed successfully");
            println!("\n✨ Your authentication has been renewed");
        }
        _ => {
            println!("ℹ️  Token refresh is only applicable for OAuth authentication");
        }
    }

    Ok(())
}
