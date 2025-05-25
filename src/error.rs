use thiserror::Error as ThisError;

/// Custom error type.
#[derive(Debug, ThisError)]
pub enum Error {
    /// API Error
    #[error("{0}")]
    APIError(#[from] reqwest::Error),
    /// IO Error
    #[error("`{0}`")]
    IOError(#[from] std::io::Error),
    /// Error may occur when handling secrets
    #[error("`{0}`")]
    KeyringError(#[from] keyring::Error),
    /// Tracker error
    #[error("`{0}`")]
    TrackerError(String),
    /// Clockify error
    #[error("{0}")]
    ClockifyError(String),
    /// Custom error
    #[error("{0}")]
    CustomError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
