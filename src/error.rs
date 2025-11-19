use thiserror::Error;

pub type ColonyResult<T> = Result<T, ColonyError>;

#[derive(Error, Debug)]
pub enum ColonyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid configuration: {0}")]
    #[allow(dead_code)]
    InvalidConfig(String),

    #[error("Parse error: {0}")]
    #[allow(dead_code)]
    Parse(String),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Colony error: {0}")]
    Colony(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("{0}")]
    Other(String),
}

impl From<anyhow::Error> for ColonyError {
    fn from(e: anyhow::Error) -> Self {
        ColonyError::Other(e.to_string())
    }
}

impl From<String> for ColonyError {
    fn from(s: String) -> Self {
        ColonyError::Other(s)
    }
}

impl From<&str> for ColonyError {
    fn from(s: &str) -> Self {
        ColonyError::Other(s.to_string())
    }
}

impl From<reqwest::Error> for ColonyError {
    fn from(e: reqwest::Error) -> Self {
        ColonyError::Auth(format!("HTTP request failed: {}", e))
    }
}
