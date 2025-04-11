use thiserror::Error as ThisError;

/// Custom error type.
#[derive(Debug, ThisError)]
pub enum Error {
    /// IO Error
    #[error("IO Error: `{0}`")]
    IO(#[from] std::io::Error),
    /// Error may occur when handling secrets
    #[error("Secrets Error: `{0}`")]
    KeyringError(#[from] keyring::Error),
    /// Custom error
    #[error("Error: `{0}`")]
    CustomError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
