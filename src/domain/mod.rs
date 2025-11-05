//! Core domain types for the vector database

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A document that has been ingested into the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Unique identifier
    pub id: Option<i64>,

    /// Source path or URL
    pub source: String,

    /// SHA-256 hash of the content for deduplication
    pub content_hash: String,

    /// Arbitrary metadata (JSON)
    pub metadata: HashMap<String, String>,

    /// Unix timestamp of creation
    pub created_at: i64,
}

impl Document {
    /// Create a new document from a source and compute its hash
    pub fn new(source: String, content: &str) -> Self {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let content_hash = format!("{:x}", hasher.finalize());

        Self {
            id: None,
            source,
            content_hash,
            metadata: HashMap::new(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        }
    }

    /// Add metadata key-value pair
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// A chunk of text from a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// Unique identifier
    pub id: Option<i64>,

    /// Parent document ID
    pub document_id: i64,

    /// Index of this chunk within the document (0-based)
    pub chunk_index: usize,

    /// The actual text content
    pub content: String,

    /// Approximate token count (for reference)
    pub token_count: Option<usize>,
}

impl Chunk {
    /// Create a new chunk
    pub fn new(document_id: i64, chunk_index: usize, content: String) -> Self {
        // Rough estimate: ~4 characters per token
        let token_count = Some(content.len() / 4);

        Self {
            id: None,
            document_id,
            chunk_index,
            content,
            token_count,
        }
    }
}

/// An embedding vector for a chunk
#[derive(Debug, Clone)]
pub struct Embedding {
    /// The chunk this embedding belongs to
    pub chunk_id: i64,

    /// Model used to generate this embedding
    pub model: String,

    /// The embedding vector
    pub vector: Vec<f32>,

    /// Dimension of the vector
    pub dimension: usize,
}

impl Embedding {
    /// Create a new embedding
    pub fn new(chunk_id: i64, model: String, vector: Vec<f32>) -> Self {
        let dimension = vector.len();
        Self {
            chunk_id,
            model,
            vector,
            dimension,
        }
    }
}

/// Result from a semantic search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// The matching chunk
    pub chunk: Chunk,

    /// The parent document
    pub document: Document,

    /// Similarity score (0.0-1.0, higher is better)
    pub similarity: f32,
}

/// Chunking strategy configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ChunkStrategy {
    /// Fixed size with overlap
    FixedSize { size: usize, overlap: usize },

    /// Semantic boundaries (sentences, paragraphs)
    Semantic { max_size: usize },
}

impl Default for ChunkStrategy {
    fn default() -> Self {
        ChunkStrategy::FixedSize {
            size: 512,
            overlap: 50,
        }
    }
}
