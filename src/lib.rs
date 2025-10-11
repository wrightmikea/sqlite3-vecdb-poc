// Vector Database CLI Library
//
// This library provides the core functionality for a semantic search system
// using SQLite's vector extension and local Ollama embedding models.

pub mod cli;
pub mod config;
pub mod domain;
pub mod error;

// Will be implemented in later phases
// pub mod services;
// pub mod repositories;
// pub mod clients;
// pub mod server;

// Re-export commonly used types
pub use error::{Result, VectDbError};
