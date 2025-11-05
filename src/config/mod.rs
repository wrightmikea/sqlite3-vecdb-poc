//! Configuration management for VectDB

use crate::domain::ChunkStrategy;
use crate::error::{Result, VectDbError};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub ollama: OllamaConfig,
    #[serde(default)]
    pub chunking: ChunkingConfig,
    #[serde(default)]
    pub search: SearchConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Path to the SQLite database file
    pub path: PathBuf,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        let path = get_default_data_dir()
            .map(|d| d.join("vectors.db"))
            .unwrap_or_else(|| PathBuf::from("vectors.db"));

        Self { path }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    /// Base URL for Ollama API
    pub base_url: String,

    /// Default embedding model
    pub default_model: String,

    /// Request timeout in seconds
    pub timeout_seconds: u64,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            default_model: "nomic-embed-text".to_string(),
            timeout_seconds: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingConfig {
    /// Maximum chunk size in characters
    pub max_chunk_size: usize,

    /// Overlap size between chunks
    pub overlap_size: usize,

    /// Chunking strategy
    #[serde(default)]
    pub strategy: String,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            max_chunk_size: 512,
            overlap_size: 50,
            strategy: "fixed".to_string(),
        }
    }
}

impl ChunkingConfig {
    /// Convert to ChunkStrategy enum
    pub fn to_strategy(&self) -> ChunkStrategy {
        match self.strategy.as_str() {
            "semantic" => ChunkStrategy::Semantic {
                max_size: self.max_chunk_size,
            },
            _ => ChunkStrategy::FixedSize {
                size: self.max_chunk_size,
                overlap: self.overlap_size,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Default number of results to return
    pub default_top_k: usize,

    /// Minimum similarity threshold
    pub similarity_threshold: f32,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            default_top_k: 10,
            similarity_threshold: 0.0,
        }
    }
}

impl Config {
    /// Load configuration from a file
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| VectDbError::Config(format!("Failed to read config file: {}", e)))?;

        let config: Config = toml::from_str(&contents)
            .map_err(|e| VectDbError::Config(format!("Failed to parse config file: {}", e)))?;

        Ok(config)
    }

    /// Load configuration with the following precedence:
    /// 1. Provided config file path
    /// 2. Default config location (~/.config/vectdb/config.toml)
    /// 3. Built-in defaults
    pub fn load(config_path: Option<PathBuf>) -> Result<Self> {
        // If explicit path provided, try to load it
        if let Some(path) = config_path {
            return Self::from_file(&path);
        }

        // Try default location
        if let Some(default_path) = get_default_config_path()
            && default_path.exists()
        {
            return Self::from_file(&default_path);
        }

        // Fall back to defaults
        Ok(Config::default())
    }

    /// Save configuration to a file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                VectDbError::Config(format!("Failed to create config directory: {}", e))
            })?;
        }

        let contents = toml::to_string_pretty(self)
            .map_err(|e| VectDbError::Config(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(path, contents)
            .map_err(|e| VectDbError::Config(format!("Failed to write config file: {}", e)))?;

        Ok(())
    }
}

/// Get the default configuration directory path
pub fn get_default_config_path() -> Option<PathBuf> {
    ProjectDirs::from("com", "vectdb", "vectdb").map(|dirs| dirs.config_dir().join("config.toml"))
}

/// Get the default data directory path
pub fn get_default_data_dir() -> Option<PathBuf> {
    ProjectDirs::from("com", "vectdb", "vectdb").map(|dirs| dirs.data_dir().to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.ollama.default_model, "nomic-embed-text");
        assert_eq!(config.search.default_top_k, 10);
    }

    #[test]
    fn test_chunking_strategy() {
        let config = ChunkingConfig::default();
        match config.to_strategy() {
            ChunkStrategy::FixedSize { size, overlap } => {
                assert_eq!(size, 512);
                assert_eq!(overlap, 50);
            }
            _ => panic!("Expected FixedSize strategy"),
        }
    }
}
