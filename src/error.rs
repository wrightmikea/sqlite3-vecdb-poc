//! Error types for the VectDB application

use thiserror::Error;

/// Main error type for VectDB operations
#[derive(Error, Debug)]
pub enum VectDbError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Ollama service unavailable: {0}")]
    OllamaUnavailable(String),

    #[error("Embedding generation failed: {0}")]
    EmbeddingFailed(String),

    #[error("Search failed: {0}")]
    SearchFailed(String),

    #[error("{0}")]
    Other(String),
}

/// Result type alias for VectDB operations
pub type Result<T> = std::result::Result<T, VectDbError>;
