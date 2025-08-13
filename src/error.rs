use thiserror::Error;

use crate::providers::ProviderError;

#[derive(Debug, Error)]
pub enum ShaidError {
    #[error("Configuration error: {0}")]
    Config(#[from] anyhow::Error),

    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to gather system context: {0}")]
    Context(String),
}

pub type Result<T> = std::result::Result<T, ShaidError>;
