use thiserror::Error;

pub type ForgeResult<T> = Result<T, ForgeError>;

#[derive(Error, Debug)]
pub enum ForgeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Server not found: {0}")]
    ServerNotFound(String),

    #[error("Claude Desktop not found. Please ensure Claude Desktop is installed.")]
    ClaudeDesktopNotFound,

    #[error("Invalid configuration: {0}")]
    #[allow(dead_code)]
    InvalidConfig(String),

    #[error("Bundle not found: {0}")]
    BundleNotFound(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Parse error: {0}")]
    #[allow(dead_code)]
    Parse(String),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Swarm error: {0}")]
    Swarm(String),

    #[error("{0}")]
    Other(String),
}

impl From<anyhow::Error> for ForgeError {
    fn from(e: anyhow::Error) -> Self {
        ForgeError::Other(e.to_string())
    }
}

impl From<String> for ForgeError {
    fn from(s: String) -> Self {
        ForgeError::Other(s)
    }
}

impl From<&str> for ForgeError {
    fn from(s: &str) -> Self {
        ForgeError::Other(s.to_string())
    }
}
