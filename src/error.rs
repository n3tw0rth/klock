use thiserror::Error;

#[derive(Debug, Error)]
pub enum JiredError {
    #[error("Auth error: {0}")]
    AuthError(String),
    #[error("Config error: {0}")]
    ConfigError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Platform error: {0}")]
    PlatformError(String),
    #[error("Session error: {0}")]
    SessionError(String),
    #[error("Not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, JiredError>;
