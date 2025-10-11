// Vector Database CLI Library
//
// This library provides the core functionality for a semantic search system
// using SQLite's vector extension and local Ollama embedding models.

pub mod cli;
pub mod clients;
pub mod config;
pub mod domain;
pub mod error;
pub mod repositories;
pub mod services;

// Will be implemented in later phases
// pub mod server;

// Re-export commonly used types
pub use clients::OllamaClient;
pub use error::{Result, VectDbError};
pub use repositories::VectorStore;
pub use services::IngestionService;
