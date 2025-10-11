//! CLI command definitions and handlers

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// VectDB - Vector Database CLI for Semantic Search
#[derive(Parser, Debug)]
#[command(name = "vectdb")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to configuration file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Set log level (error, warn, info, debug, trace)
    #[arg(short, long, default_value = "info")]
    pub log_level: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize VectDB configuration
    Init {
        /// Force overwrite existing configuration
        #[arg(short, long)]
        force: bool,
    },

    /// Ingest documents into the vector database
    Ingest {
        /// Source file or directory path
        source: PathBuf,

        /// Embedding model to use (e.g., nomic-embed-text)
        #[arg(short, long, default_value = "nomic-embed-text")]
        model: String,

        /// Maximum chunk size in tokens
        #[arg(short = 's', long, default_value = "512")]
        chunk_size: usize,

        /// Overlap size between chunks
        #[arg(short = 'o', long, default_value = "50")]
        overlap: usize,

        /// Process directories recursively
        #[arg(short, long)]
        recursive: bool,
    },

    /// Search the vector database
    Search {
        /// Search query
        query: String,

        /// Number of results to return
        #[arg(short = 'k', long, default_value = "10")]
        top_k: usize,

        /// Similarity threshold (0.0-1.0)
        #[arg(short = 't', long, default_value = "0.0")]
        threshold: f32,

        /// Show detailed similarity scores
        #[arg(short = 'e', long)]
        explain: bool,

        /// Output format (text, json, csv)
        #[arg(short = 'f', long, default_value = "text")]
        format: String,
    },

    /// Start the web server
    Serve {
        /// Server port
        #[arg(short, long, default_value = "3000")]
        port: u16,

        /// Server host
        #[arg(short = 'H', long, default_value = "127.0.0.1")]
        host: String,
    },

    /// Show database statistics
    Stats,

    /// Optimize database (vacuum and analyze)
    Optimize,

    /// List available Ollama models
    Models,
}
