use std::io;
use thiserror::Error;

/// Unified error type for the ShellGPT application
#[derive(Error, Debug)]
pub enum SchatError {
    /// API-related errors (OpenAI, OpenRouter, etc.)
    #[error("API error: {0}")]
    Api(String),

    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// User input errors
    #[error("Input error: {0}")]
    Input(String),

    /// Command execution errors
    #[error("Execution error: {0}")]
    Execution(String),

    /// IO-related errors
    #[error("IO error: {source}")]
    Io {
        #[from]
        source: io::Error,
    },

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Network-related errors
    #[error("Network error: {0}")]
    Network(String),

    /// Unknown or unexpected errors
    #[error("Unknown error: {0}")]
    Unknown(String),

    /// Tool-related errors
    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    /// Tool execution errors
    #[error("Tool execution error: {0}")]
    ToolExecution(String),

    /// MCP connection errors
    #[error("MCP connection error: {0}")]
    McpConnection(String),
}

impl From<reqwest::Error> for SchatError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            SchatError::Network(format!("Request timed out: {}", err))
        } else if err.is_connect() {
            SchatError::Network(format!("Connection failed: {}", err))
        } else if err.is_status() {
            SchatError::Api(format!("API returned error status: {}", err))
        } else {
            SchatError::Network(format!("Request failed: {}", err))
        }
    }
}

impl From<serde_json::Error> for SchatError {
    fn from(err: serde_json::Error) -> Self {
        SchatError::Serialization(format!("JSON error: {}", err))
    }
}

impl From<serde_yml::Error> for SchatError {
    fn from(err: serde_yml::Error) -> Self {
        SchatError::Serialization(format!("YAML error: {}", err))
    }
}

impl From<String> for SchatError {
    fn from(err: String) -> Self {
        SchatError::Unknown(err)
    }
}

impl From<&str> for SchatError {
    fn from(err: &str) -> Self {
        SchatError::Unknown(err.to_string())
    }
}

impl From<Box<dyn std::error::Error>> for SchatError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        SchatError::Unknown(err.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for SchatError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        SchatError::Unknown(err.to_string())
    }
}
