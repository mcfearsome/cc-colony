use crate::config::{ForgeConfig, McpServer};
use crate::error::{ForgeError, ForgeResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
    token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub description: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub version: String,
    pub author: Option<String>,
    pub repository: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    pub name: String,
    pub description: String,
    pub servers: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub servers: Vec<ServerInfo>,
    pub total: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BundleListResponse {
    pub bundles: Vec<Bundle>,
    pub total: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub recommended_servers: Vec<ServerInfo>,
    pub recommended_bundles: Vec<Bundle>,
    pub updates: Vec<UpdateInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub server_name: String,
    pub current_version: String,
    pub latest_version: String,
    pub changelog: Option<String>,
}

impl ApiClient {
    pub fn new() -> ForgeResult<Self> {
        let config = ForgeConfig::load()?;

        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        Ok(Self {
            client,
            base_url: config.api_url,
            token: config.api_token,
        })
    }

    #[allow(dead_code)]
    pub fn with_token(token: String) -> ForgeResult<Self> {
        let config = ForgeConfig::load()?;

        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;

        Ok(Self {
            client,
            base_url: config.api_url,
            token: Some(token),
        })
    }

    fn auth_header(&self) -> ForgeResult<String> {
        self.token.clone().ok_or_else(|| {
            ForgeError::Auth("Not authenticated. Please run 'forge login'.".to_string())
        })
    }

    /// Login and get authentication token
    pub async fn login(&self, email: String, password: String) -> ForgeResult<LoginResponse> {
        let response = self
            .client
            .post(format!("{}/api/auth/login", self.base_url))
            .json(&LoginRequest { email, password })
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(ForgeError::Auth(format!(
                "Login failed: {}",
                response.status()
            )));
        }

        let login_response: LoginResponse = response.json().await?;
        Ok(login_response)
    }

    /// Search for MCP servers
    pub async fn search_servers(&self, query: Option<String>) -> ForgeResult<Vec<ServerInfo>> {
        let mut url = format!("{}/api/servers/search", self.base_url);

        if let Some(q) = query {
            url = format!("{}?q={}", url, urlencoding::encode(&q));
        }

        let mut request = self.client.get(&url);

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(ForgeError::Network(format!(
                "Search failed: {}",
                response.status()
            )));
        }

        let search_response: SearchResponse = response.json().await?;
        Ok(search_response.servers)
    }

    /// Get server details by name
    pub async fn get_server(&self, name: &str) -> ForgeResult<ServerInfo> {
        let url = format!("{}/api/servers/{}", self.base_url, name);

        let mut request = self.client.get(&url);

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if response.status().is_client_error() {
            return Err(ForgeError::ServerNotFound(name.to_string()));
        }

        if !response.status().is_success() {
            return Err(ForgeError::Network(format!(
                "Failed to get server: {}",
                response.status()
            )));
        }

        let server: ServerInfo = response.json().await?;
        Ok(server)
    }

    /// List available bundles
    pub async fn list_bundles(&self) -> ForgeResult<Vec<Bundle>> {
        let url = format!("{}/api/bundles", self.base_url);

        let mut request = self.client.get(&url);

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(ForgeError::Network(format!(
                "Failed to list bundles: {}",
                response.status()
            )));
        }

        let bundle_response: BundleListResponse = response.json().await?;
        Ok(bundle_response.bundles)
    }

    /// Get bundle details by name
    pub async fn get_bundle(&self, name: &str) -> ForgeResult<Bundle> {
        let url = format!("{}/api/bundles/{}", self.base_url, name);

        let mut request = self.client.get(&url);

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        if response.status().is_client_error() {
            return Err(ForgeError::BundleNotFound(name.to_string()));
        }

        if !response.status().is_success() {
            return Err(ForgeError::Network(format!(
                "Failed to get bundle: {}",
                response.status()
            )));
        }

        let bundle: Bundle = response.json().await?;
        Ok(bundle)
    }

    /// Sync with cloud to get recommendations
    pub async fn sync(&self) -> ForgeResult<SyncResponse> {
        let token = self.auth_header()?;
        let url = format!("{}/api/sync", self.base_url);

        let response = self.client.get(&url).bearer_auth(token).send().await?;

        if !response.status().is_success() {
            return Err(ForgeError::Network(format!(
                "Sync failed: {}",
                response.status()
            )));
        }

        let sync_response: SyncResponse = response.json().await?;
        Ok(sync_response)
    }
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            client: Client::new(),
            base_url: "https://api.useforge.cc".to_string(),
            token: None,
        })
    }
}

impl ServerInfo {
    pub fn to_mcp_server(&self) -> McpServer {
        McpServer {
            command: self.command.clone(),
            args: self.args.clone(),
            env: self.env.clone(),
            other: HashMap::new(),
        }
    }
}
