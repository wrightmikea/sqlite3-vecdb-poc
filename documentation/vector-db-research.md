# Vector Database CLI Research Document

## Executive Summary

This document outlines the architecture, requirements, and implementation plan for a Rust-based CLI application that demonstrates semantic search capabilities using SQLite's vector extension (sqlite-vec), local Ollama embedding models, and a web-based UI for visualization and interaction.

## Product Requirements Document (PRD)

### Vision Statement

Create a self-contained, reproducible demonstration of vector database technology that allows users to build semantic search systems using local resources, emphasizing transparency, educational value, and practical utility.

### Core Functional Requirements

1. **Text Ingestion**
   - Download text files from URLs or local filesystem
   - Support common text formats (txt, md, pdf, html)
   - Chunk text intelligently (semantic boundaries, configurable size)
   - Store original text with metadata (source, timestamp, checksum)

2. **Embedding Generation**
   - Interface with local Ollama API for embedding generation
   - Support multiple embedding models (configurable)
   - Batch processing for efficiency
   - Progress tracking and resumability

3. **Vector Storage**
   - Persistent SQLite database with vector extension
   - Efficient vector similarity search (cosine, euclidean)
   - Metadata indexing for hybrid search
   - Schema versioning for upgradability

4. **Query Interface**
   - CLI commands for semantic search
   - Configurable result ranking (top-k, similarity threshold)
   - Explain mode showing similarity scores
   - Export results to various formats (JSON, CSV, markdown)

5. **Web UI**
   - Interactive search interface
   - Visualization of document relationships
   - Corpus statistics and health metrics
   - Real-time ingestion status

### User Stories

- As a researcher, I want to quickly find relevant passages across multiple documents using natural language queries
- As a developer, I want to understand how vector databases work through a transparent, inspectable implementation
- As a data engineer, I want to validate embedding quality and search accuracy through the web UI
- As a student, I want to explore semantic relationships in my document collection

### Non-Functional Requirements

#### Availability
- Graceful degradation when Ollama service unavailable
- Local-first architecture (no external dependencies beyond Ollama)
- Database corruption recovery mechanisms

#### Reliability
- Transactional consistency for all database operations
- Checkpointing for long-running ingestion tasks
- Comprehensive error handling with actionable messages

#### Usability
- Intuitive CLI with helpful defaults and examples
- Progressive disclosure of advanced features
- Clear feedback for all operations
- Comprehensive help documentation

#### Maintainability
- Modular architecture with clear separation of concerns
- Extensive inline documentation and literate programming style
- Comprehensive test coverage (unit, integration, property-based)
- Code formatting and linting enforcement

#### Discoverability
- Self-documenting CLI with --help for all commands
- Interactive tutorials and examples in web UI
- Schema introspection capabilities
- Logging with multiple verbosity levels

#### Serviceability
- Detailed logging with structured output (JSON option)
- Database vacuum and optimization commands
- Health check endpoint in web server
- Performance profiling hooks

#### Traceability
- Request IDs for all operations
- Audit log of database modifications
- Reproducible builds with locked dependencies
- Git-trackable configuration files

#### Upgradability
- Database schema migrations
- Backward-compatible CLI interface
- Configuration version detection
- Safe rollback procedures

#### Performance
- Streaming ingestion for large files
- Parallel embedding generation
- Efficient vector index (HNSW or IVF)
- Response time targets: <100ms for queries, <1s for ingestion per chunk

#### Utility
- Practical for real-world document collections (1K-100K documents)
- Support for incremental updates
- De-duplication of content
- Export/import capabilities for corpus sharing

#### Understandability
- Clear architectural documentation
- Design rationale for key decisions
- Example use cases with expected outcomes
- Visualization of system behavior

## Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                         User Layer                          │
│  ┌──────────────┐                    ┌──────────────┐      │
│  │  CLI Client  │                    │   Web UI     │      │
│  │   (Clap)     │                    │(HTML/CSS/JS) │      │
│  └──────┬───────┘                    └──────┬───────┘      │
└─────────┼────────────────────────────────────┼─────────────┘
          │                                    │
┌─────────┼────────────────────────────────────┼─────────────┐
│         │         Application Layer          │             │
│  ┌──────▼───────┐                    ┌──────▼───────┐     │
│  │   CLI Core   │◄───────────────────┤  HTTP Server │     │
│  │   Handler    │                    │   (Axum)     │     │
│  └──────┬───────┘                    └──────┬───────┘     │
│         │                                    │             │
│  ┌──────▼────────────────────────────────────▼───────┐    │
│  │           Business Logic Layer                    │    │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐       │    │
│  │  │ Ingestion│  │  Search  │  │Analytics │       │    │
│  │  │ Service  │  │  Service │  │ Service  │       │    │
│  │  └────┬─────┘  └────┬─────┘  └────┬─────┘       │    │
│  └───────┼─────────────┼─────────────┼──────────────┘    │
└──────────┼─────────────┼─────────────┼───────────────────┘
           │             │             │
┌──────────┼─────────────┼─────────────┼───────────────────┐
│          │    Data Access Layer      │                   │
│  ┌───────▼──────┐          ┌─────────▼──────┐           │
│  │   Embedding  │          │  Vector Store  │           │
│  │   Repository │          │   Repository   │           │
│  └───────┬──────┘          └────────┬───────┘           │
└──────────┼──────────────────────────┼───────────────────┘
           │                          │
┌──────────┼──────────────────────────┼───────────────────┐
│          │  Infrastructure Layer    │                   │
│  ┌───────▼──────┐          ┌────────▼───────┐          │
│  │    Ollama    │          │  SQLite + Vec  │          │
│  │   API Client │          │    Database    │          │
│  │  (HTTP/REST) │          │  (rusqlite)    │          │
│  └──────────────┘          └────────────────┘          │
└──────────────────────────────────────────────────────────┘
```

### Architectural Patterns

#### Hexagonal Architecture (Ports & Adapters)
- Core domain logic isolated from infrastructure
- Ports define interfaces for external interactions
- Adapters implement specific technologies (SQLite, Ollama, HTTP)
- Enables testing with mock implementations

#### Repository Pattern
- Abstract data access behind well-defined interfaces
- Swap storage implementations without changing business logic
- Simplifies testing with in-memory repositories

#### Command Pattern (CLI)
- Each CLI command is a self-contained unit
- Composable and testable independently
- Clear separation of parsing, validation, and execution

### Design Rationale

#### Why SQLite?
- **Zero configuration**: Single-file database, no server setup
- **Portability**: Database file is fully portable across platforms
- **Reliability**: ACID compliance, battle-tested
- **Embeddable**: Perfect for CLI applications
- **sqlite-vec extension**: Efficient vector operations with minimal dependencies

#### Why Ollama?
- **Local-first**: No external API dependencies or costs
- **Privacy**: Data never leaves the local machine
- **Flexibility**: Support for multiple embedding models
- **Standardized API**: REST interface simplifies integration
- **Active development**: Growing ecosystem and model support

#### Why Rust?
- **Performance**: Near C-level performance for intensive operations
- **Safety**: Memory safety without garbage collection
- **Concurrency**: Fearless concurrency for parallel processing
- **Ecosystem**: Excellent libraries (rusqlite, reqwest, clap, axum)
- **Binary size**: Single compiled binary for easy distribution

#### Why Separate Web UI?
- **Flexibility**: Users can choose CLI-only or full-featured UI
- **Development velocity**: Iterate on UI without recompiling Rust
- **Accessibility**: Web technologies are widely understood
- **Visualization**: Rich interactive visualizations easier in browser
- **Optional deployment**: Can run headless in production

## High Level Design

### Component Descriptions

#### 1. CLI Handler (`src/cli/mod.rs`)
**Responsibility**: Parse commands, validate inputs, orchestrate business logic

**Interface**:
```rust
pub enum Command {
    Ingest {
        source: PathBuf,
        model: String,
        chunk_size: usize,
    },
    Search {
        query: String,
        top_k: usize,
        threshold: f32,
    },
    Serve {
        port: u16,
        host: String,
    },
    Stats,
    Optimize,
}

pub async fn execute(cmd: Command, config: Config) -> Result<()>;
```

#### 2. Ingestion Service (`src/services/ingestion.rs`)
**Responsibility**: Load documents, chunk text, coordinate embedding generation

**Key Methods**:
- `ingest_file(path: &Path) -> Result<Document>`
- `chunk_document(doc: &Document, strategy: ChunkStrategy) -> Vec<Chunk>`
- `generate_embeddings(chunks: &[Chunk], model: &str) -> Result<Vec<Embedding>>`
- `store_with_embeddings(chunks: Vec<Chunk>, embeddings: Vec<Embedding>) -> Result<()>`

**Chunking Strategies**:
- Fixed size with overlap (configurable)
- Sentence boundary aware
- Semantic section detection (markdown headers, paragraph breaks)

#### 3. Search Service (`src/services/search.rs`)
**Responsibility**: Execute semantic queries, rank results, hybrid search

**Key Methods**:
- `semantic_search(query: &str, top_k: usize) -> Result<Vec<SearchResult>>`
- `hybrid_search(query: &str, filters: Metadata) -> Result<Vec<SearchResult>>`
- `explain_search(query: &str, result: &SearchResult) -> Explanation`

**Ranking Strategies**:
- Pure vector similarity (cosine/euclidean)
- Hybrid with BM25 text search
- Metadata boosting (recency, source trust)

#### 4. Vector Store Repository (`src/repositories/vector_store.rs`)
**Responsibility**: Abstract vector database operations

**Schema**:
```sql
CREATE TABLE documents (
    id INTEGER PRIMARY KEY,
    source TEXT NOT NULL,
    content_hash TEXT UNIQUE NOT NULL,
    metadata TEXT, -- JSON
    created_at INTEGER NOT NULL
);

CREATE TABLE chunks (
    id INTEGER PRIMARY KEY,
    document_id INTEGER NOT NULL,
    chunk_index INTEGER NOT NULL,
    content TEXT NOT NULL,
    token_count INTEGER,
    FOREIGN KEY (document_id) REFERENCES documents(id)
);

CREATE TABLE embeddings (
    chunk_id INTEGER PRIMARY KEY,
    model TEXT NOT NULL,
    vector BLOB NOT NULL, -- Float32 array
    dimension INTEGER NOT NULL,
    FOREIGN KEY (chunk_id) REFERENCES chunks(id)
);

CREATE INDEX idx_chunks_document ON chunks(document_id);
CREATE INDEX idx_embeddings_model ON embeddings(model);
```

**Vector Index**:
```sql
-- Using sqlite-vec for efficient similarity search
CREATE VIRTUAL TABLE vec_index USING vec0(
    chunk_id INTEGER PRIMARY KEY,
    embedding float[768]  -- Dimension depends on model
);
```

#### 5. Ollama Client (`src/clients/ollama.rs`)
**Responsibility**: Interface with Ollama API

**Interface**:
```rust
pub struct OllamaClient {
    base_url: String,
    client: reqwest::Client,
}

impl OllamaClient {
    pub async fn embed(&self, model: &str, texts: &[String]) 
        -> Result<Vec<Vec<f32>>>;
    
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>>;
    
    pub async fn health_check(&self) -> Result<bool>;
}
```

**Error Handling**:
- Retry with exponential backoff for transient failures
- Clear error messages for model not found
- Graceful degradation if Ollama unavailable

#### 6. Web Server (`src/server/mod.rs`)
**Responsibility**: Serve HTTP API and static web UI

**Endpoints**:
```
GET  /                      -> Serve web UI
GET  /api/search            -> Semantic search
POST /api/ingest            -> Trigger ingestion
GET  /api/stats             -> Corpus statistics
GET  /api/documents         -> List documents
GET  /api/documents/:id     -> Get document details
GET  /api/health            -> Health check
WS   /api/ws/ingest         -> Real-time ingestion status
```

### Data Flow

#### Ingestion Flow
```
1. User: vectdb ingest document.txt --model nomic-embed-text
2. CLI parses command → Ingestion Service
3. Load file → Validate format → Hash content
4. Check for duplicates in Vector Store
5. Chunk document using configured strategy
6. Batch chunks → Ollama Client
7. For each batch:
   - Request embeddings from Ollama
   - Retry on failure with backoff
   - Store chunk + embedding in transaction
8. Update document metadata
9. Report success with stats
```

#### Search Flow
```
1. User: vectdb search "rust async programming"
2. CLI/Web → Search Service
3. Generate query embedding via Ollama
4. Vector Store: similarity search using sqlite-vec
5. Optional: Apply metadata filters
6. Rank results by similarity
7. Retrieve original chunks + metadata
8. Format and return results
```

### State Management

**Configuration State** (`~/.vectdb/config.toml`):
```toml
[database]
path = "~/.vectdb/vectors.db"

[ollama]
base_url = "http://localhost:11434"
default_model = "nomic-embed-text"
timeout_seconds = 30

[chunking]
strategy = "semantic"
max_chunk_size = 512
overlap_size = 50

[search]
default_top_k = 10
similarity_threshold = 0.7
```

**Runtime State**:
- Database connection pool (managed by rusqlite)
- Ollama client (persistent HTTP connection)
- Web server state (Axum shared state)

## Implementation Plan

### Phase 1: Foundation (Week 1)

**Goals**: Establish project structure, core abstractions, basic CLI

**Tasks**:
1. Initialize Rust project with workspace structure
2. Define core domain types (Document, Chunk, Embedding)
3. Implement configuration loading (config-rs)
4. Create CLI skeleton with clap
5. Set up logging infrastructure (tracing)
6. Write project documentation (README, CONTRIBUTING)

**Deliverables**:
- Compiling project with basic CLI
- Configuration file schema
- Initial test suite structure

**Acceptance Criteria**:
- `cargo build` succeeds
- `vectdb --help` displays all commands
- Configuration loads from default and custom paths

### Phase 2: Vector Store (Week 2)

**Goals**: Implement SQLite database layer with vector support

**Tasks**:
1. Set up rusqlite with sqlite-vec extension
2. Implement schema migrations (refinery)
3. Create Vector Store repository interface
4. Implement CRUD operations for documents/chunks
5. Add vector insertion and similarity search
6. Write comprehensive repository tests

**Deliverables**:
- Working Vector Store with full CRUD
- Migration system
- Benchmark suite for search performance

**Acceptance Criteria**:
- Store 1000 documents with embeddings
- Search completes in <100ms for 10K vectors
- Migrations apply cleanly
- Test coverage >80%

### Phase 3: Ollama Integration (Week 2-3)

**Goals**: Build robust Ollama API client

**Tasks**:
1. Implement HTTP client with reqwest
2. Add embedding generation endpoint
3. Implement retry logic and error handling
4. Add model listing and validation
5. Create mock Ollama server for testing
6. Write integration tests

**Deliverables**:
- Ollama client library
- Mock server for CI/CD
- Integration test suite

**Acceptance Criteria**:
- Generate embeddings for batch of 100 texts
- Handle Ollama service unavailable gracefully
- Retry transient failures automatically
- Mock tests run without actual Ollama instance

### Phase 4: Ingestion Pipeline (Week 3-4)

**Goals**: Complete end-to-end ingestion workflow

**Tasks**:
1. Implement text file loaders (txt, markdown, PDF)
2. Create chunking strategies
3. Build ingestion service orchestrator
4. Add progress tracking and resumability
5. Implement duplicate detection
6. Add batch processing with concurrency control

**Deliverables**:
- Working `vectdb ingest` command
- Support for multiple file formats
- Progress indicators

**Acceptance Criteria**:
- Ingest 100 documents successfully
- Resume interrupted ingestion
- Detect and skip duplicate content
- Chunk large files without memory issues

### Phase 5: Search Implementation (Week 4)

**Goals**: Implement semantic search with ranking

**Tasks**:
1. Build search service
2. Implement vector similarity search
3. Add result ranking and filtering
4. Create explain mode for debugging
5. Add metadata-based hybrid search
6. Performance optimization

**Deliverables**:
- Working `vectdb search` command
- Multiple ranking strategies
- Explain mode for transparency

**Acceptance Criteria**:
- Search returns relevant results
- Top-k filtering works correctly
- Explain mode shows similarity scores
- Search latency <100ms for 10K vectors

### Phase 6: Web Server & UI (Week 5)

**Goals**: Build HTTP API and interactive web interface

**Tasks**:
1. Set up Axum web server
2. Implement REST API endpoints
3. Create static web UI (HTML/CSS/JavaScript)
4. Add WebSocket for real-time updates
5. Build search interface with visualizations
6. Add corpus statistics dashboard

**Deliverables**:
- Working `vectdb serve` command
- Interactive web UI
- Real-time ingestion monitoring

**Acceptance Criteria**:
- Web UI accessible at localhost:3000
- Search works through web interface
- WebSocket shows live ingestion progress
- Corpus statistics display correctly

### Phase 7: Polish & Documentation (Week 6)

**Goals**: Finalize features, comprehensive documentation, examples

**Tasks**:
1. Write comprehensive README with examples
2. Create user guide and tutorials
3. Add example datasets
4. Performance benchmarking and optimization
5. Security audit and hardening
6. Create demo video/GIF for README

**Deliverables**:
- Complete documentation
- Example projects
- Performance benchmarks
- Release-ready binary

**Acceptance Criteria**:
- Documentation covers all features
- Example runs successfully for new users
- Benchmarks meet performance targets
- Security best practices followed

### Testing Strategy

**Unit Tests**:
- All pure functions and business logic
- Mock external dependencies (DB, HTTP)
- Property-based testing for chunking algorithms (proptest)

**Integration Tests**:
- Database operations with real SQLite
- Ollama client with mock server
- End-to-end CLI commands

**Performance Tests**:
- Vector search at scale (10K, 100K, 1M vectors)
- Ingestion throughput
- Memory usage profiling

**Acceptance Tests**:
- User scenarios from requirements
- Example use cases from documentation

### Deployment & Distribution

**Binary Distribution**:
- GitHub Releases with pre-built binaries
- Cross-compilation for Linux, macOS, Windows
- Docker image for containerized deployment
- Homebrew formula for macOS

**Database Initialization**:
- Automatic schema creation on first run
- Migration warnings for schema updates
- Backup recommendations in documentation

## Technology Stack

### Core Dependencies

**CLI & Configuration**:
- `clap` (4.x) - Command-line argument parsing
- `config` - Configuration management
- `directories` - Cross-platform config/data directories

**Database**:
- `rusqlite` (0.31+) - SQLite bindings
- `sqlite-vec` - Vector similarity search extension
- `refinery` - Database migrations

**HTTP & Networking**:
- `axum` (0.7+) - Web framework
- `tower` - Middleware for Axum
- `reqwest` (0.12+) - HTTP client for Ollama
- `tokio` (1.x) - Async runtime

**Data Processing**:
- `serde` / `serde_json` - Serialization
- `regex` - Text processing
- `sha2` - Content hashing
- `unicode-segmentation` - Text chunking

**Observability**:
- `tracing` / `tracing-subscriber` - Structured logging
- `metrics` - Performance metrics

**Testing**:
- `proptest` - Property-based testing
- `wiremock` - HTTP mocking
- `tempfile` - Temporary test databases

### Web UI Stack

**Core**:
- Vanilla JavaScript (ES6+) - No framework overhead
- CSS Grid/Flexbox - Responsive layout
- Web Components (optional) - Encapsulation

**Visualization**:
- D3.js - Graph visualizations
- Plotly.js - Statistical charts
- Highlight.js - Code/text highlighting

**Optional Enhancements**:
- WASM module for client-side embedding (future)
- Service Worker for offline capability
- IndexedDB for client-side caching

## Risk Mitigation

### Technical Risks

**Risk**: sqlite-vec performance insufficient at scale
- **Mitigation**: Benchmark early, fallback to external vector DB (Qdrant) if needed
- **Detection**: Performance tests in Phase 2

**Risk**: Ollama API changes breaking compatibility
- **Mitigation**: Version pinning, adapter pattern for easy switching
- **Detection**: Integration tests, monitoring Ollama releases

**Risk**: Memory issues with large documents
- **Mitigation**: Streaming ingestion, chunking, memory profiling
- **Detection**: Integration tests with large files

### Operational Risks

**Risk**: Database corruption from crashes
- **Mitigation**: SQLite WAL mode, regular backups, recovery procedures
- **Detection**: Automated integrity checks

**Risk**: Ollama service unavailable
- **Mitigation**: Queue system, graceful degradation, clear error messages
- **Detection**: Health checks, monitoring

## Future Enhancements

**Phase 8+**:
- Multi-modal embeddings (images, audio)
- Distributed vector store for scaling
- Fine-tuning evaluation framework
- Export to standard formats (FAISS, HDF5)
- Plugin system for custom loaders
- GraphQL API option
- Collaborative features (shared collections)

## Success Metrics

**Technical**:
- Search latency <100ms (p95) for 10K documents
- Ingestion throughput >100 chunks/second
- Memory usage <500MB for 100K vectors
- Test coverage >80%

**Usability**:
- First successful search within 5 minutes of installation
- Clear error messages (no cryptic failures)
- Documentation completeness (all features covered)

**Educational**:
- Users understand vector database concepts after using tool
- Code examples reusable in other projects
- Architecture decisions well-documented

---

## Appendix: Example Usage

### Basic Workflow
```bash
# Initialize configuration
vectdb init

# Ingest documents
vectdb ingest ./documents/ --model nomic-embed-text --recursive

# Search
vectdb search "explain async/await in rust" --top-k 5

# Start web UI
vectdb serve --port 3000

# View statistics
vectdb stats

# Optimize database
vectdb optimize
```

### Advanced Usage
```bash
# Custom chunking
vectdb ingest book.txt --chunk-size 256 --overlap 32

# Hybrid search with filters
vectdb search "optimization techniques" \
  --metadata "source:rust-book" \
  --threshold 0.8

# Export results
vectdb search "memory safety" --format json > results.json

# Database maintenance
vectdb vacuum
vectdb migrate --target 2
```

---

*Document Version*: 1.0  
*Last Updated*: 2025-10-11  
*Status*: Ready for Implementation