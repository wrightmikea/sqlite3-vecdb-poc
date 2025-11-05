//! Vector Store repository implementation
//!
//! Provides database operations for documents, chunks, and embeddings using SQLite.

use crate::domain::{Chunk, Document, Embedding, SearchResult};
use crate::error::Result;
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;
use tracing::{debug, info};

/// Vector Store manages all database operations
pub struct VectorStore {
    conn: Connection,
}

impl VectorStore {
    /// Create a new VectorStore with a connection to the database
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        info!("Opening database at: {:?}", db_path.as_ref());

        let conn = Connection::open(db_path)?;

        // Enable WAL mode for better concurrency
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", true)?;

        let mut store = Self { conn };
        store.init_schema()?;

        Ok(store)
    }

    /// Create an in-memory database (useful for testing)
    pub fn in_memory() -> Result<Self> {
        info!("Creating in-memory database");

        let conn = Connection::open_in_memory()?;
        conn.pragma_update(None, "foreign_keys", true)?;

        let mut store = Self { conn };
        store.init_schema()?;

        Ok(store)
    }

    /// Initialize the database schema
    fn init_schema(&mut self) -> Result<()> {
        info!("Initializing database schema");

        // Create documents table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS documents (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source TEXT NOT NULL,
                content_hash TEXT UNIQUE NOT NULL,
                metadata TEXT,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Create chunks table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS chunks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                document_id INTEGER NOT NULL,
                chunk_index INTEGER NOT NULL,
                content TEXT NOT NULL,
                token_count INTEGER,
                FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
                UNIQUE(document_id, chunk_index)
            )",
            [],
        )?;

        // Create embeddings table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS embeddings (
                chunk_id INTEGER PRIMARY KEY,
                model TEXT NOT NULL,
                vector BLOB NOT NULL,
                dimension INTEGER NOT NULL,
                FOREIGN KEY (chunk_id) REFERENCES chunks(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create indices
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_chunks_document ON chunks(document_id)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_embeddings_model ON embeddings(model)",
            [],
        )?;

        info!("Schema initialized successfully");
        Ok(())
    }

    // ============================================================================
    // Document Operations
    // ============================================================================

    /// Insert a new document
    pub fn insert_document(&mut self, doc: &Document) -> Result<i64> {
        debug!("Inserting document: {}", doc.source);

        let metadata_json = serde_json::to_string(&doc.metadata)?;

        self.conn.execute(
            "INSERT INTO documents (source, content_hash, metadata, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                &doc.source,
                &doc.content_hash,
                &metadata_json,
                doc.created_at
            ],
        )?;

        let id = self.conn.last_insert_rowid();
        info!("Inserted document with id: {}", id);

        Ok(id)
    }

    /// Get a document by ID
    pub fn get_document(&self, id: i64) -> Result<Option<Document>> {
        debug!("Getting document with id: {}", id);

        let result = self
            .conn
            .query_row(
                "SELECT id, source, content_hash, metadata, created_at FROM documents WHERE id = ?1",
                params![id],
                |row| {
                    let metadata_json: String = row.get(3)?;
                    let metadata = serde_json::from_str(&metadata_json)
                        .unwrap_or_default();

                    Ok(Document {
                        id: Some(row.get(0)?),
                        source: row.get(1)?,
                        content_hash: row.get(2)?,
                        metadata,
                        created_at: row.get(4)?,
                    })
                },
            )
            .optional()?;

        Ok(result)
    }

    /// Get a document by content hash (for deduplication)
    pub fn get_document_by_hash(&self, content_hash: &str) -> Result<Option<Document>> {
        debug!("Getting document by hash: {}", content_hash);

        let result = self
            .conn
            .query_row(
                "SELECT id, source, content_hash, metadata, created_at FROM documents
                 WHERE content_hash = ?1",
                params![content_hash],
                |row| {
                    let metadata_json: String = row.get(3)?;
                    let metadata = serde_json::from_str(&metadata_json).unwrap_or_default();

                    Ok(Document {
                        id: Some(row.get(0)?),
                        source: row.get(1)?,
                        content_hash: row.get(2)?,
                        metadata,
                        created_at: row.get(4)?,
                    })
                },
            )
            .optional()?;

        Ok(result)
    }

    /// Count total documents
    pub fn count_documents(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))?;

        Ok(count)
    }

    // ============================================================================
    // Chunk Operations
    // ============================================================================

    /// Insert a new chunk
    pub fn insert_chunk(&mut self, chunk: &Chunk) -> Result<i64> {
        debug!(
            "Inserting chunk {} for document {}",
            chunk.chunk_index, chunk.document_id
        );

        self.conn.execute(
            "INSERT INTO chunks (document_id, chunk_index, content, token_count)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                chunk.document_id,
                chunk.chunk_index,
                &chunk.content,
                chunk.token_count
            ],
        )?;

        let id = self.conn.last_insert_rowid();
        Ok(id)
    }

    /// Get all chunks for a document
    pub fn get_chunks_for_document(&self, document_id: i64) -> Result<Vec<Chunk>> {
        debug!("Getting chunks for document {}", document_id);

        let mut stmt = self.conn.prepare(
            "SELECT id, document_id, chunk_index, content, token_count
             FROM chunks
             WHERE document_id = ?1
             ORDER BY chunk_index",
        )?;

        let chunks = stmt
            .query_map(params![document_id], |row| {
                Ok(Chunk {
                    id: Some(row.get(0)?),
                    document_id: row.get(1)?,
                    chunk_index: row.get(2)?,
                    content: row.get(3)?,
                    token_count: row.get(4)?,
                })
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(chunks)
    }

    /// Get a chunk by ID
    pub fn get_chunk(&self, id: i64) -> Result<Option<Chunk>> {
        debug!("Getting chunk with id: {}", id);

        let result = self
            .conn
            .query_row(
                "SELECT id, document_id, chunk_index, content, token_count FROM chunks WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Chunk {
                        id: Some(row.get(0)?),
                        document_id: row.get(1)?,
                        chunk_index: row.get(2)?,
                        content: row.get(3)?,
                        token_count: row.get(4)?,
                    })
                },
            )
            .optional()?;

        Ok(result)
    }

    /// Count total chunks
    pub fn count_chunks(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM chunks", [], |row| row.get(0))?;

        Ok(count)
    }

    // ============================================================================
    // Embedding Operations
    // ============================================================================

    /// Insert or update an embedding for a chunk
    pub fn upsert_embedding(&mut self, embedding: &Embedding) -> Result<()> {
        debug!("Upserting embedding for chunk {}", embedding.chunk_id);

        // Convert vector to bytes
        let vector_bytes = vector_to_bytes(&embedding.vector);

        self.conn.execute(
            "INSERT OR REPLACE INTO embeddings (chunk_id, model, vector, dimension)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                embedding.chunk_id,
                &embedding.model,
                &vector_bytes,
                embedding.dimension
            ],
        )?;

        Ok(())
    }

    /// Get an embedding for a chunk
    pub fn get_embedding(&self, chunk_id: i64) -> Result<Option<Embedding>> {
        debug!("Getting embedding for chunk {}", chunk_id);

        let result = self
            .conn
            .query_row(
                "SELECT chunk_id, model, vector, dimension FROM embeddings WHERE chunk_id = ?1",
                params![chunk_id],
                |row| {
                    let vector_bytes: Vec<u8> = row.get(2)?;
                    let vector = bytes_to_vector(&vector_bytes);

                    Ok(Embedding {
                        chunk_id: row.get(0)?,
                        model: row.get(1)?,
                        vector,
                        dimension: row.get(3)?,
                    })
                },
            )
            .optional()?;

        Ok(result)
    }

    /// Count total embeddings
    pub fn count_embeddings(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM embeddings", [], |row| row.get(0))?;

        Ok(count)
    }

    // ============================================================================
    // Search Operations (Placeholder for now - will use sqlite-vec in future)
    // ============================================================================

    /// Search for similar vectors (naive implementation using cosine similarity)
    ///
    /// Note: This is a placeholder implementation. In Phase 2b, we'll integrate
    /// sqlite-vec for efficient vector similarity search using HNSW or IVF indices.
    pub fn search_similar(
        &self,
        query_vector: &[f32],
        model: &str,
        top_k: usize,
    ) -> Result<Vec<SearchResult>> {
        debug!("Searching for similar vectors (top_k={})", top_k);

        // Get all embeddings for the specified model
        let mut stmt = self.conn.prepare(
            "SELECT e.chunk_id, e.model, e.vector, e.dimension,
                    c.id, c.document_id, c.chunk_index, c.content, c.token_count,
                    d.id, d.source, d.content_hash, d.metadata, d.created_at
             FROM embeddings e
             JOIN chunks c ON e.chunk_id = c.id
             JOIN documents d ON c.document_id = d.id
             WHERE e.model = ?1",
        )?;

        let mut results: Vec<(f32, SearchResult)> = stmt
            .query_map(params![model], |row| {
                // Parse embedding
                let vector_bytes: Vec<u8> = row.get(2)?;
                let vector = bytes_to_vector(&vector_bytes);

                // Calculate cosine similarity
                let similarity = cosine_similarity(query_vector, &vector);

                // Parse chunk
                let chunk = Chunk {
                    id: Some(row.get(4)?),
                    document_id: row.get(5)?,
                    chunk_index: row.get(6)?,
                    content: row.get(7)?,
                    token_count: row.get(8)?,
                };

                // Parse document
                let metadata_json: String = row.get(12)?;
                let metadata = serde_json::from_str(&metadata_json).unwrap_or_default();

                let document = Document {
                    id: Some(row.get(9)?),
                    source: row.get(10)?,
                    content_hash: row.get(11)?,
                    metadata,
                    created_at: row.get(13)?,
                };

                Ok((
                    similarity,
                    SearchResult {
                        chunk,
                        document,
                        similarity,
                    },
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        // Sort by similarity (descending) and take top k
        results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);

        let search_results = results.into_iter().map(|(_, result)| result).collect();

        Ok(search_results)
    }

    // ============================================================================
    // Database Maintenance
    // ============================================================================

    /// Run VACUUM to optimize database size
    pub fn vacuum(&self) -> Result<()> {
        info!("Running VACUUM on database");
        self.conn.execute("VACUUM", [])?;
        Ok(())
    }

    /// Run ANALYZE to update query optimizer statistics
    pub fn analyze(&self) -> Result<()> {
        info!("Running ANALYZE on database");
        self.conn.execute("ANALYZE", [])?;
        Ok(())
    }

    /// Get database statistics
    pub fn get_stats(&self) -> Result<DatabaseStats> {
        let doc_count = self.count_documents()?;
        let chunk_count = self.count_chunks()?;
        let embedding_count = self.count_embeddings()?;

        // Get database file size
        let page_count: i64 = self
            .conn
            .query_row("PRAGMA page_count", [], |row| row.get(0))?;
        let page_size: i64 = self
            .conn
            .query_row("PRAGMA page_size", [], |row| row.get(0))?;
        let db_size_bytes = page_count * page_size;

        Ok(DatabaseStats {
            document_count: doc_count,
            chunk_count,
            embedding_count,
            db_size_bytes,
        })
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub document_count: i64,
    pub chunk_count: i64,
    pub embedding_count: i64,
    pub db_size_bytes: i64,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Convert a vector of f32 to bytes (little-endian)
fn vector_to_bytes(vector: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(vector.len() * 4);
    for value in vector {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

/// Convert bytes to a vector of f32 (little-endian)
fn bytes_to_vector(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| {
            let array: [u8; 4] = chunk.try_into().unwrap();
            f32::from_le_bytes(array)
        })
        .collect()
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }

    dot_product / (magnitude_a * magnitude_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_conversion() {
        let original = vec![1.0, 2.5, -3.15, 0.0];
        let bytes = vector_to_bytes(&original);
        let converted = bytes_to_vector(&bytes);

        assert_eq!(original.len(), converted.len());
        for (a, b) in original.iter().zip(converted.iter()) {
            assert!((a - b).abs() < 0.0001);
        }
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.0001);

        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&a, &b).abs() < 0.0001);

        let a = vec![1.0, 1.0];
        let b = vec![1.0, 1.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_database_creation() {
        let store = VectorStore::in_memory().unwrap();
        let stats = store.get_stats().unwrap();

        assert_eq!(stats.document_count, 0);
        assert_eq!(stats.chunk_count, 0);
        assert_eq!(stats.embedding_count, 0);
    }

    #[test]
    fn test_document_operations() {
        let mut store = VectorStore::in_memory().unwrap();

        let doc = Document::new("test.txt".to_string(), "Hello world");
        let doc_id = store.insert_document(&doc).unwrap();

        assert!(doc_id > 0);

        let retrieved = store.get_document(doc_id).unwrap().unwrap();
        assert_eq!(retrieved.source, "test.txt");
        assert_eq!(retrieved.content_hash, doc.content_hash);

        let count = store.count_documents().unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_chunk_operations() {
        let mut store = VectorStore::in_memory().unwrap();

        let doc = Document::new("test.txt".to_string(), "Hello world");
        let doc_id = store.insert_document(&doc).unwrap();

        let chunk = Chunk::new(doc_id, 0, "Hello world".to_string());
        let chunk_id = store.insert_chunk(&chunk).unwrap();

        assert!(chunk_id > 0);

        let retrieved = store.get_chunk(chunk_id).unwrap().unwrap();
        assert_eq!(retrieved.content, "Hello world");
        assert_eq!(retrieved.chunk_index, 0);

        let chunks = store.get_chunks_for_document(doc_id).unwrap();
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn test_embedding_operations() {
        let mut store = VectorStore::in_memory().unwrap();

        let doc = Document::new("test.txt".to_string(), "Hello world");
        let doc_id = store.insert_document(&doc).unwrap();

        let chunk = Chunk::new(doc_id, 0, "Hello world".to_string());
        let chunk_id = store.insert_chunk(&chunk).unwrap();

        let vector = vec![0.1, 0.2, 0.3];
        let embedding = Embedding::new(chunk_id, "test-model".to_string(), vector.clone());

        store.upsert_embedding(&embedding).unwrap();

        let retrieved = store.get_embedding(chunk_id).unwrap().unwrap();
        assert_eq!(retrieved.model, "test-model");
        assert_eq!(retrieved.dimension, 3);
        assert_eq!(retrieved.vector, vector);
    }

    #[test]
    fn test_search_similar() {
        let mut store = VectorStore::in_memory().unwrap();

        // Insert test data
        let doc = Document::new("test.txt".to_string(), "Test document");
        let doc_id = store.insert_document(&doc).unwrap();

        let chunk1 = Chunk::new(doc_id, 0, "First chunk".to_string());
        let chunk1_id = store.insert_chunk(&chunk1).unwrap();

        let chunk2 = Chunk::new(doc_id, 1, "Second chunk".to_string());
        let chunk2_id = store.insert_chunk(&chunk2).unwrap();

        // Insert embeddings
        let embedding1 = Embedding::new(chunk1_id, "model".to_string(), vec![1.0, 0.0, 0.0]);
        store.upsert_embedding(&embedding1).unwrap();

        let embedding2 = Embedding::new(chunk2_id, "model".to_string(), vec![0.0, 1.0, 0.0]);
        store.upsert_embedding(&embedding2).unwrap();

        // Search with query similar to first embedding
        let query = vec![0.9, 0.1, 0.0];
        let results = store.search_similar(&query, "model", 2).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].chunk.content, "First chunk");
        assert!(results[0].similarity > results[1].similarity);
    }
}
