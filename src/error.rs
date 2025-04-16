use thiserror::Error as ThisError;

/// Custom error type.
#[derive(Debug, ThisError)]
pub enum Error {
    /// API Error
    #[error("API Error: `{0}`")]
    APIError(#[from] reqwest::Error),
    /// IO Error
    #[error("IO Error: `{0}`")]
    IOError(#[from] std::io::Error),
    /// Error may occur when handling secrets
    #[error("Secrets Error: `{0}`")]
    KeyringError(#[from] keyring::Error),
    /// Tracker error
    #[error("Error: `{0}`")]
    TrackerError(String),
    /// Custom error
    #[error("Error: `{0}`")]
    CustomError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
