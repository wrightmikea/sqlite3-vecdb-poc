//! Search service for semantic queries
//!
//! Provides semantic search functionality using embeddings and vector similarity.

use crate::clients::OllamaClient;
use crate::domain::SearchResult;
use crate::error::Result;
use crate::repositories::VectorStore;
use tracing::{debug, info};

/// Service for performing semantic searches
pub struct SearchService {
    store: VectorStore,
    ollama: OllamaClient,
}

impl SearchService {
    /// Create a new search service
    pub fn new(store: VectorStore, ollama: OllamaClient) -> Self {
        Self { store, ollama }
    }

    /// Perform a semantic search
    pub async fn search(
        &self,
        query: &str,
        model: &str,
        top_k: usize,
        threshold: f32,
    ) -> Result<Vec<SearchResult>> {
        info!("Performing semantic search: query='{}', top_k={}, threshold={}", query, top_k, threshold);

        // Generate embedding for the query
        debug!("Generating query embedding");
        let query_embedding = self.ollama.embed(model, query).await?;

        // Search for similar vectors
        debug!("Searching for similar vectors");
        let mut results = self.store.search_similar(&query_embedding, model, top_k)?;

        // Filter by threshold
        if threshold > 0.0 {
            results.retain(|r| r.similarity >= threshold);
            debug!("Filtered to {} results above threshold {}", results.len(), threshold);
        }

        info!("Found {} results", results.len());

        Ok(results)
    }
}

/// Format search results as text
pub fn format_results_text(results: &[SearchResult], explain: bool) -> String {
    if results.is_empty() {
        return "No results found.".to_string();
    }

    let mut output = String::new();

    output.push_str(&format!("Found {} result(s):\n\n", results.len()));

    for (idx, result) in results.iter().enumerate() {
        output.push_str(&format!("=== Result {} ===\n", idx + 1));

        if explain {
            output.push_str(&format!("Similarity: {:.4}\n", result.similarity));
        }

        output.push_str(&format!("Source: {}\n", result.document.source));
        output.push_str(&format!("Chunk {}\n\n", result.chunk.chunk_index + 1));

        // Truncate long content for display
        let content = if result.chunk.content.len() > 500 {
            format!("{}...", &result.chunk.content[..500])
        } else {
            result.chunk.content.clone()
        };

        output.push_str(&format!("{}\n\n", content));
    }

    output
}

/// Format search results as JSON
pub fn format_results_json(results: &[SearchResult]) -> Result<String> {
    let json = serde_json::to_string_pretty(results)?;
    Ok(json)
}

/// Format search results as CSV
pub fn format_results_csv(results: &[SearchResult]) -> String {
    let mut output = String::new();

    // Header
    output.push_str("rank,similarity,source,chunk_index,content\n");

    // Rows
    for (idx, result) in results.iter().enumerate() {
        let content = result.chunk.content.replace('"', "\"\""); // Escape quotes
        let content = content.replace('\n', " "); // Remove newlines

        output.push_str(&format!(
            "{},{:.4},\"{}\",{},\"{}\"\n",
            idx + 1,
            result.similarity,
            result.document.source,
            result.chunk.chunk_index + 1,
            content
        ));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::domain::{Chunk, Document};

    #[test]
    fn test_format_results_text_empty() {
        let results = vec![];
        let output = format_results_text(&results, false);
        assert!(output.contains("No results found"));
    }

    #[test]
    fn test_format_results_text() {
        let doc = Document::new("test.txt".to_string(), "test content");
        let chunk = Chunk::new(1, 0, "Test chunk content".to_string());
        let result = SearchResult {
            chunk,
            document: doc,
            similarity: 0.95,
        };

        let output = format_results_text(&[result], true);
        assert!(output.contains("Result 1"));
        assert!(output.contains("0.95"));
        assert!(output.contains("test.txt"));
        assert!(output.contains("Test chunk content"));
    }

    #[test]
    fn test_format_results_json() {
        let doc = Document::new("test.txt".to_string(), "test content");
        let chunk = Chunk::new(1, 0, "Test chunk".to_string());
        let result = SearchResult {
            chunk,
            document: doc,
            similarity: 0.85,
        };

        let output = format_results_json(&[result]).unwrap();
        assert!(output.contains("test.txt"));
        assert!(output.contains("0.85"));
    }

    #[test]
    fn test_format_results_csv() {
        let doc = Document::new("test.txt".to_string(), "test content");
        let chunk = Chunk::new(1, 0, "Test chunk".to_string());
        let result = SearchResult {
            chunk,
            document: doc,
            similarity: 0.75,
        };

        let output = format_results_csv(&[result]);
        assert!(output.contains("rank,similarity,source"));
        assert!(output.contains("0.75"));
        assert!(output.contains("test.txt"));
    }

    #[test]
    fn test_format_csv_escapes_quotes() {
        let doc = Document::new("test.txt".to_string(), "test");
        let chunk = Chunk::new(1, 0, "Content with \"quotes\"".to_string());
        let result = SearchResult {
            chunk,
            document: doc,
            similarity: 0.5,
        };

        let output = format_results_csv(&[result]);
        assert!(output.contains("\"\""));
    }
}
