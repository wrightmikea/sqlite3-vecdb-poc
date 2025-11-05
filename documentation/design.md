# VectDB Design Document

## Overview

This document describes the detailed design of VectDB's key components, data structures, algorithms, and interfaces.

## Core Data Structures

### Document

Represents an ingested document with metadata and deduplication support.

```rust
pub struct Document {
    pub id: Option<i64>,           // Auto-increment primary key
    pub source: String,             // File path or URL
    pub content_hash: String,       // SHA-256 hash for deduplication
    pub metadata: HashMap<String, String>,  // Arbitrary key-value pairs
    pub created_at: i64,            // Unix timestamp
}
```

**Design Decisions**:
- SHA-256 provides strong deduplication guarantees
- Metadata as HashMap allows flexible extension
- Unix timestamp for cross-platform compatibility

### Chunk

Represents a text segment from a document.

```rust
pub struct Chunk {
    pub id: Option<i64>,            // Auto-increment primary key
    pub document_id: i64,           // Foreign key to document
    pub chunk_index: usize,         // Position within document (0-based)
    pub content: String,            // Actual text content
    pub token_count: Option<usize>, // Approximate token count
}
```

**Design Decisions**:
- chunk_index preserves document structure
- token_count is approximate (4 chars ≈ 1 token heuristic)
- String content allows efficient access without re-parsing

### Embedding

Represents a vector embedding for a chunk.

```rust
pub struct Embedding {
    pub chunk_id: i64,              // Foreign key to chunk (also PK)
    pub model: String,              // e.g., "nomic-embed-text"
    pub vector: Vec<f32>,           // Embedding vector
    pub dimension: usize,           // Vector dimensionality
}
```

**Design Decisions**:
- chunk_id as primary key (one embedding per chunk-model pair)
- model field allows multiple embeddings per chunk (future)
- Vec<f32> for IEEE 754 compatibility
- dimension for validation and metadata

## Algorithms

### Cosine Similarity

Used for comparing query and document embeddings.

```rust
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }

    dot_product / (magnitude_a * magnitude_b)
}
```

**Properties**:
- Range: [0, 1] (we use positive embeddings)
- Symmetric: sim(a, b) = sim(b, a)
- Computationally efficient: O(n) where n = dimension

**Alternatives Considered**:
- Euclidean distance: Less common for embeddings
- Dot product: Not normalized, scale-dependent
- Manhattan distance: Less accurate for high-dimensional spaces

### Text Chunking

Two strategies implemented:

#### Fixed-Size Chunking

```rust
fn chunk_fixed_size(text: &str, size: usize, overlap: usize) -> Vec<String> {
    // Split on grapheme clusters (Unicode-aware)
    // Window size: 'size' graphemes
    // Step size: 'size - overlap' graphemes
    // Handles edge cases: empty text, size <= overlap
}
```

**Parameters**:
- `size`: Number of characters/graphemes per chunk
- `overlap`: Number of overlapping characters between adjacent chunks

**Rationale**: Overlap ensures context continuity across chunk boundaries.

#### Semantic Chunking

```rust
fn chunk_semantic(text: &str, max_size: usize) -> Vec<String> {
    // 1. Split on paragraph boundaries (\n\n)
    // 2. Within paragraphs, split on sentence boundaries (., !, ?)
    // 3. If sentence > max_size, fall back to fixed-size chunking
    // 4. Accumulate sentences until max_size reached
}
```

**Rationale**: Preserves semantic units (sentences, paragraphs) for better retrieval quality.

### Content Hashing

SHA-256 used for deduplication:

```rust
use sha2::{Digest, Sha256};

let mut hasher = Sha256::new();
hasher.update(content.as_bytes());
let content_hash = format!("{:x}", hasher.finalize());
```

**Properties**:
- Deterministic: Same content → same hash
- Collision-resistant: Practically impossible to find two documents with same hash
- Fast: ~500 MB/s on modern hardware

## Component Interfaces

### VectorStore

```rust
pub struct VectorStore {
    conn: Connection,  // SQLite connection
}

impl VectorStore {
    // Lifecycle
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self>;
    pub fn in_memory() -> Result<Self>;

    // Document operations
    pub fn insert_document(&self, doc: &Document) -> Result<i64>;
    pub fn find_by_hash(&self, hash: &str) -> Result<Option<Document>>;
    pub fn get_document(&self, id: i64) -> Result<Option<Document>>;
    pub fn delete_document(&self, id: i64) -> Result<()>;

    // Chunk operations
    pub fn insert_chunk(&self, chunk: &Chunk) -> Result<i64>;
    pub fn get_chunks_by_document(&self, doc_id: i64) -> Result<Vec<Chunk>>;

    // Embedding operations
    pub fn upsert_embedding(&self, embedding: &Embedding) -> Result<()>;
    pub fn get_embedding(&self, chunk_id: i64, model: &str) -> Result<Option<Embedding>>;

    // Search operations
    pub fn search_similar(&self, query_vector: &[f32], model: &str, top_k: usize) -> Result<Vec<SearchResult>>;

    // Statistics
    pub fn get_stats(&self) -> Result<DatabaseStats>;
}
```

### OllamaClient

```rust
pub struct OllamaClient {
    base_url: String,
    client: reqwest::Client,
    timeout: Duration,
}

impl OllamaClient {
    pub fn new(base_url: String, timeout_seconds: u64) -> Result<Self>;

    // Health check
    pub async fn health_check(&self) -> Result<bool>;

    // Embedding generation
    pub async fn embed(&self, text: &str, model: &str) -> Result<Vec<f32>>;
    pub async fn embed_batch(&self, texts: Vec<String>, model: &str) -> Result<Vec<Vec<f32>>>;
    pub async fn embed_with_retry(&self, text: &str, model: &str, max_retries: u32) -> Result<Vec<f32>>;

    // Model management
    pub async fn list_models(&self) -> Result<Vec<String>>;
}
```

### IngestionService

```rust
pub struct IngestionService<'a> {
    store: &'a VectorStore,
    ollama: &'a OllamaClient,
}

impl<'a> IngestionService<'a> {
    pub fn new(store: &'a VectorStore, ollama: &'a OllamaClient) -> Self;

    pub async fn ingest_file(
        &self,
        path: &Path,
        model: &str,
        strategy: ChunkStrategy,
    ) -> Result<IngestionResult>;

    pub async fn load_file(path: &Path) -> Result<String>;
}

pub struct IngestionResult {
    pub document_id: i64,
    pub chunks_created: usize,
    pub embeddings_created: usize,
    pub skipped: bool,
}
```

### SearchService

```rust
pub struct SearchService<'a> {
    store: &'a VectorStore,
    ollama: &'a OllamaClient,
}

impl<'a> SearchService<'a> {
    pub fn new(store: &'a VectorStore, ollama: &'a OllamaClient) -> Self;

    pub async fn search(
        &self,
        query: &str,
        model: &str,
        top_k: usize,
        threshold: f32,
    ) -> Result<Vec<SearchResult>>;
}

// Formatting functions
pub fn format_results_text(results: &[SearchResult], show_scores: bool) -> String;
pub fn format_results_json(results: &[SearchResult]) -> Result<String>;
pub fn format_results_csv(results: &[SearchResult]) -> Result<String>;
```

## Database Design

### Schema Versioning

Currently no versioning (v1). Future migrations will use:
- `PRAGMA user_version` for schema version tracking
- Migration scripts in `migrations/` directory
- Automatic upgrade on connection

### Query Patterns

**Insert Document**:
```sql
INSERT INTO documents (source, content_hash, metadata, created_at)
VALUES (?, ?, ?, ?)
```

**Find Duplicate**:
```sql
SELECT * FROM documents WHERE content_hash = ?
```

**Search Embeddings**:
```sql
SELECT c.id, c.content, c.document_id, c.chunk_index,
       d.source, d.metadata,
       e.vector
FROM embeddings e
JOIN chunks c ON e.chunk_id = c.id
JOIN documents d ON c.document_id = d.id
WHERE e.model = ?
```

**Get Statistics**:
```sql
SELECT
    (SELECT COUNT(*) FROM documents) as doc_count,
    (SELECT COUNT(*) FROM chunks) as chunk_count,
    (SELECT COUNT(*) FROM embeddings) as embedding_count
```

## Error Handling

### Error Types

```rust
pub enum VectDbError {
    Database(rusqlite::Error),
    Http(reqwest::Error),
    Io(std::io::Error),
    Config(String),
    Serialization(serde_json::Error),
    Other(String),
}
```

### Error Propagation

- Use `?` operator for automatic conversion
- Wrap external errors with context
- Return `Result<T>` for all fallible operations

### Error Recovery

- **Ollama Unavailable**: Return graceful error, suggest starting service
- **File Not Found**: Skip and continue processing (in batch operations)
- **Parse Errors**: Log warning, skip malformed data
- **Database Locked**: Retry with exponential backoff (SQLite handles this)

## Configuration

### File Format (TOML)

```toml
[database]
path = "~/.local/share/vectdb/vectors.db"

[ollama]
base_url = "http://localhost:11434"
default_model = "nomic-embed-text"
timeout_seconds = 30

[chunking]
max_chunk_size = 512
overlap_size = 50
strategy = "fixed"  # or "semantic"

[search]
default_top_k = 10
similarity_threshold = 0.0
```

### Configuration Precedence

1. Command-line arguments (highest priority)
2. Config file specified via `--config`
3. Default config location (`~/.config/vectdb/config.toml`)
4. Built-in defaults (lowest priority)

## Web Server Design

### Architecture

```
HTTP Request
    ↓
Axum Router
    ↓
Handler Function
    ↓
AppState (Config + OllamaClient)
    ↓
Create VectorStore ──→ Search/Stats
    ↓
Format Response
    ↓
HTTP Response
```

### Endpoints

| Method | Path | Description | Status Codes |
|--------|------|-------------|--------------|
| GET | `/` | HTML UI | 200 |
| GET | `/build-info.js` | Build metadata | 200 |
| GET | `/favicon.ico` | Favicon | 200, 404 |
| GET | `/api/health` | Service health | 200 |
| GET | `/api/stats` | DB statistics | 200 |
| GET | `/api/search` | Semantic search | 200, 400, 500 |
| GET | `/api/models` | List models | 200, 503 |

### State Management

```rust
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub ollama: Arc<OllamaClient>,  // Shared across requests
}
```

**Design Decision**: VectorStore is NOT shared (created per-request) because rusqlite::Connection is !Send+!Sync.

## WebAssembly Demo

### Architecture

```
Browser
    ↓
index.html (loads WASM)
    ↓
vectdb_demo.js (JS bindings)
    ↓
vectdb_demo_bg.wasm (Rust code)
    ↓
Canned Data (compiled into WASM)
```

### Exposed Functions

```rust
#[wasm_bindgen]
pub fn search_demo(query: &str) -> String;  // Returns JSON

#[wasm_bindgen]
pub fn get_stats_demo() -> String;  // Returns JSON
```

### Build Process

```bash
cd demo
wasm-pack build --target web --out-dir ../docs --release
```

**Optimizations**:
- `opt-level = "z"` (optimize for size)
- `lto = true` (link-time optimization)
- `wasm-opt` (run by wasm-pack)

Result: ~20KB WASM binary

## Performance Considerations

### Ingestion

- **Batch embeddings**: 10-50 chunks per request to Ollama
- **Transaction boundaries**: Commit per document (not per chunk)
- **Parallel processing**: Could use rayon for CPU-bound chunking

### Search

- **Pre-filter by model**: Avoid loading wrong embeddings
- **Top-K heap**: Use BinaryHeap for efficient k-largest selection
- **Lazy loading**: Don't load document content until needed

### Memory Usage

- **Streaming**: Read files in chunks if > 10MB
- **Connection pooling**: Not needed (SQLite handles it)
- **Vector caching**: Consider LRU cache for hot vectors

## Security Considerations

### Input Validation

- **Path Traversal**: Validate file paths, resolve symlinks
- **SQL Injection**: Use parameterized queries (rusqlite handles this)
- **Content Limits**: Enforce max file size (e.g., 100MB)
- **Rate Limiting**: Add to web server if exposed publicly

### Content Safety

- **No Eval**: Never execute user-provided code
- **Sanitize Metadata**: Escape HTML in web UI
- **Content Hashing**: Prevent tampering detection

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        // Test orthogonal, parallel, opposite vectors
    }

    #[tokio::test]
    async fn test_ingest_file() {
        // Use in-memory VectorStore
        // Mock or skip Ollama calls
    }
}
```

### Integration Tests

```bash
# tests/integration_test.rs
# End-to-end workflows
```

### Property-Based Testing

Consider quickcheck for:
- Chunking strategies (no empty chunks, proper overlap)
- Similarity calculations (symmetric, bounded)

## References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [SQLite Best Practices](https://www.sqlite.org/quickstart.html)
- [Vector Search Algorithms](https://www.pinecone.io/learn/vector-search/)
