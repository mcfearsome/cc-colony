pub mod oauth;
pub mod providers;
pub mod token_store;

use serde::{Deserialize, Serialize};
use crate::error::ColonyResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "kebab-case")]
pub enum AuthProvider {
    /// Direct API key
    #[serde(rename = "api-key")]
    ApiKey {
        #[serde(skip_serializing_if = "Option::is_none")]
        api_key: Option<String>,
    },

    /// AWS Bedrock
    #[serde(rename = "bedrock")]
    Bedrock {
        region: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        profile: Option<String>,
    },

    /// OAuth for Claude.ai Pro/Max users
    #[serde(rename = "anthropic-oauth")]
    AnthropicOAuth {
        token_path: String,
    },

    /// Google Cloud Vertex AI
    #[serde(rename = "vertex-ai")]
    VertexAI {
        project: String,
        location: String,
    },
}

impl Default for AuthProvider {
    fn default() -> Self {
        AuthProvider::ApiKey { api_key: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    #[serde(flatten)]
    pub provider: AuthProvider,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            provider: AuthProvider::ApiKey { api_key: None },
        }
    }
}

pub struct AuthManager {
    provider: AuthProvider,
}

impl AuthManager {
    pub fn new(provider: AuthProvider) -> Self {
        Self { provider }
    }

    /// Get access token/credentials for API calls
    pub async fn get_credentials(&self) -> ColonyResult<AuthCredentials> {
        match &self.provider {
            AuthProvider::ApiKey { api_key } => {
                let key = if let Some(k) = api_key {
                    k.clone()
                } else {
                    // Try environment variable
                    std::env::var("ANTHROPIC_API_KEY")
                        .map_err(|_| crate::error::ColonyError::Auth(
                            "No API key configured. Set ANTHROPIC_API_KEY or configure in colony.yml".to_string()
                        ))?
                };

                Ok(AuthCredentials::ApiKey(key))
            }

            AuthProvider::Bedrock { region, profile } => {
                Ok(AuthCredentials::Bedrock {
                    region: region.clone(),
                    profile: profile.clone(),
                })
            }

            AuthProvider::AnthropicOAuth { token_path } => {
                let token_store = token_store::TokenStore::new(token_path.into());
                let token = token_store.load_token()?
                    .ok_or_else(|| crate::error::ColonyError::Auth(
                        "Not authenticated. Run 'colony auth login' first".to_string()
                    ))?;

                // Check if token is expired
                if token.is_expired() {
                    return Err(crate::error::ColonyError::Auth(
                        "Token expired. Run 'colony auth refresh'".to_string()
                    ));
                }

                Ok(AuthCredentials::OAuth(token.access_token))
            }

            AuthProvider::VertexAI { project, location } => {
                Ok(AuthCredentials::VertexAI {
                    project: project.clone(),
                    location: location.clone(),
                })
            }
        }
    }

    /// Validate current authentication
    pub async fn validate(&self) -> ColonyResult<bool> {
        // Test API call to verify auth works
        match self.get_credentials().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AuthCredentials {
    ApiKey(String),
    OAuth(String),
    Bedrock {
        region: String,
        profile: Option<String>,
    },
    VertexAI {
        project: String,
        location: String,
    },
}
