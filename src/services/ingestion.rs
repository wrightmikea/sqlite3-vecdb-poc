//! Ingestion service for processing documents
//!
//! Handles loading files, chunking text, generating embeddings, and storing in the database.

use crate::clients::OllamaClient;
use crate::domain::{Chunk, ChunkStrategy, Document, Embedding};
use crate::error::{Result, VectDbError};
use crate::repositories::VectorStore;
use crate::services::chunking::chunk_text;
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

/// Service for ingesting documents into the vector database
pub struct IngestionService {
    store: VectorStore,
    ollama: OllamaClient,
}

impl IngestionService {
    /// Create a new ingestion service
    pub fn new(store: VectorStore, ollama: OllamaClient) -> Self {
        Self { store, ollama }
    }

    /// Ingest a single file
    pub async fn ingest_file(
        &mut self,
        file_path: &Path,
        model: &str,
        strategy: ChunkStrategy,
    ) -> Result<IngestionResult> {
        info!("Ingesting file: {:?}", file_path);

        // Load file content
        let content = self.load_file(file_path)?;

        if content.trim().is_empty() {
            warn!("File is empty, skipping: {:?}", file_path);
            return Ok(IngestionResult {
                file_path: file_path.to_path_buf(),
                document_id: 0,
                chunks_created: 0,
                embeddings_created: 0,
                skipped: true,
            });
        }

        // Create document
        let source = file_path.to_string_lossy().to_string();
        let document = Document::new(source, &content);

        // Check for duplicates
        if let Some(existing) = self.store.get_document_by_hash(&document.content_hash)? {
            info!("Document already exists (duplicate content), skipping: {:?}", file_path);
            return Ok(IngestionResult {
                file_path: file_path.to_path_buf(),
                document_id: existing.id.unwrap_or(0),
                chunks_created: 0,
                embeddings_created: 0,
                skipped: true,
            });
        }

        // Insert document
        let document_id = self.store.insert_document(&document)?;
        info!("Created document with ID: {}", document_id);

        // Chunk the text
        let chunk_texts = chunk_text(&content, strategy);
        info!("Created {} chunks", chunk_texts.len());

        // Create and insert chunks
        let mut chunk_ids = Vec::new();
        for (idx, chunk_text) in chunk_texts.iter().enumerate() {
            let chunk = Chunk::new(document_id, idx, chunk_text.clone());
            let chunk_id = self.store.insert_chunk(&chunk)?;
            chunk_ids.push(chunk_id);
        }

        debug!("Inserted {} chunks into database", chunk_ids.len());

        // Generate embeddings
        info!("Generating embeddings using model: {}", model);
        let embeddings = self.ollama.embed_batch(model, &chunk_texts).await?;

        if embeddings.len() != chunk_ids.len() {
            return Err(VectDbError::EmbeddingFailed(format!(
                "Expected {} embeddings but got {}",
                chunk_ids.len(),
                embeddings.len()
            )));
        }

        // Store embeddings
        for (chunk_id, embedding_vec) in chunk_ids.iter().zip(embeddings.iter()) {
            let embedding = Embedding::new(*chunk_id, model.to_string(), embedding_vec.clone());
            self.store.upsert_embedding(&embedding)?;
        }

        info!("Successfully ingested {:?}", file_path);

        Ok(IngestionResult {
            file_path: file_path.to_path_buf(),
            document_id,
            chunks_created: chunk_ids.len(),
            embeddings_created: embeddings.len(),
            skipped: false,
        })
    }

    /// Ingest multiple files
    pub async fn ingest_files(
        &mut self,
        file_paths: &[impl AsRef<Path>],
        model: &str,
        strategy: ChunkStrategy,
    ) -> Result<Vec<IngestionResult>> {
        let mut results = Vec::new();

        for file_path in file_paths {
            match self.ingest_file(file_path.as_ref(), model, strategy).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    warn!("Failed to ingest {:?}: {}", file_path.as_ref(), e);
                    results.push(IngestionResult {
                        file_path: file_path.as_ref().to_path_buf(),
                        document_id: 0,
                        chunks_created: 0,
                        embeddings_created: 0,
                        skipped: true,
                    });
                }
            }
        }

        Ok(results)
    }

    /// Load file content (supports txt and md for now)
    fn load_file(&self, file_path: &Path) -> Result<String> {
        debug!("Loading file: {:?}", file_path);

        if !file_path.exists() {
            return Err(VectDbError::InvalidInput(format!(
                "File does not exist: {:?}",
                file_path
            )));
        }

        if !file_path.is_file() {
            return Err(VectDbError::InvalidInput(format!(
                "Path is not a file: {:?}",
                file_path
            )));
        }

        // Check file extension
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match extension.as_str() {
            "txt" | "md" | "markdown" => {
                let content = fs::read_to_string(file_path)?;
                Ok(content)
            }
            "" => {
                // Try to read as text anyway
                let content = fs::read_to_string(file_path)?;
                Ok(content)
            }
            _ => Err(VectDbError::InvalidInput(format!(
                "Unsupported file type: .{}. Currently supported: txt, md",
                extension
            ))),
        }
    }
}

/// Result of ingesting a file
#[derive(Debug, Clone)]
pub struct IngestionResult {
    pub file_path: std::path::PathBuf,
    pub document_id: i64,
    pub chunks_created: usize,
    pub embeddings_created: usize,
    pub skipped: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_load_file_txt() {
        let config = Config::default();
        let store = VectorStore::in_memory().unwrap();
        let ollama = OllamaClient::new(config.ollama.base_url, config.ollama.timeout_seconds).unwrap();
        let service = IngestionService::new(store, ollama);

        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Hello, world!").unwrap();

        let content = service.load_file(temp_file.path()).unwrap();
        assert!(content.contains("Hello, world!"));
    }

    #[test]
    fn test_load_file_nonexistent() {
        let config = Config::default();
        let store = VectorStore::in_memory().unwrap();
        let ollama = OllamaClient::new(config.ollama.base_url, config.ollama.timeout_seconds).unwrap();
        let service = IngestionService::new(store, ollama);

        let result = service.load_file(Path::new("/nonexistent/file.txt"));
        assert!(result.is_err());
    }
}
