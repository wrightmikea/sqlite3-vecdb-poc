# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

VectDB is a Rust-based CLI application for semantic search using SQLite's vector extension (sqlite-vec) and local Ollama embedding models. It's a local-first, privacy-focused vector database that enables semantic search across document collections without external API dependencies.

**Current Status**: Phase 6 complete (Web Server & REST API), Phase 7 in progress (Polish & Documentation)

## Development Process - IMPORTANT

**Before making any changes**, review the [Development Process](documentation/process.md) documentation.

### Required Pre-Commit Quality Checks

**EVERY commit must pass these checks** (no exceptions):

1. **Format code**: `cargo fmt`
   - Verify: `cargo fmt -- --check`

2. **Fix all Clippy warnings**: `cargo clippy --all-targets --all-features -- -D warnings`
   - Must have zero warnings (Rust 2024 edition has stricter lints)

3. **Run all tests**: `cargo test`
   - All tests must pass

4. **Build release**: `cargo build --release`
   - Must compile without errors or warnings

5. **Verify .gitignore**: Check `git status` for unexpected files
   - Ensure build artifacts excluded (target/, *.log, *.db)
   - Protect document directories (texts/, documents/, data/)

6. **Update documentation**: If changes affect user-facing features or architecture
   - Update README.md, CLAUDE.md, or documentation/*.md as needed

### Test-Driven Development (TDD)

Use the **Red-Green-Refactor** cycle for all features and fixes:

1. **Red**: Write failing test first (defines expected behavior)
2. **Green**: Write minimal code to make test pass
3. **Refactor**: Improve code quality while keeping tests green

See [documentation/process.md](documentation/process.md) for detailed TDD workflow and examples.

### UI Testing

For web interface changes, use **MCP/Playwright**:
```bash
# Start server in background
./scripts/serve.sh &

# Use MCP/Playwright tools to:
# - Navigate to pages
# - Fill forms and click buttons
# - Verify results
# - Take screenshots

# Stop server when done
```

### Comprehensive Documentation

The `documentation/` directory contains detailed guides:
- **[architecture.md](documentation/architecture.md)** - System architecture and design patterns
- **[prd.md](documentation/prd.md)** - Product requirements and user stories
- **[design.md](documentation/design.md)** - Component designs and algorithms
- **[process.md](documentation/process.md)** - Development workflow (TDD, quality checks)
- **[status.md](documentation/status.md)** - Current status and metrics
- **[plan.md](documentation/plan.md)** - Roadmap and next steps

**Consult these documents before starting work on any significant feature or refactoring.**

## Build & Test Commands

### Building
```bash
# Development build
cargo build

# Release build
cargo build --release

# The binary is at target/release/vectdb
```

### Testing
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests for a specific module
cargo test vector_store
```

### Running the CLI
```bash
# Development
cargo run -- <command> [options]

# With logging
cargo run -- --log-level debug <command>
RUST_LOG=trace cargo run -- <command>

# Examples
cargo run -- init
cargo run -- ingest texts/ -r
cargo run -- search "query text" -k 5
cargo run -- serve -p 3000
cargo run -- stats
```

## Architecture

VectDB follows a **hexagonal (ports and adapters) architecture** with clear separation of concerns:

### Module Structure

```
src/
├── main.rs              # Entry point, CLI dispatch, command handlers
├── lib.rs               # Public library API
├── error.rs             # Error types (VectDbError, Result alias)
├── cli/                 # CLI definitions (clap commands)
├── config/              # Configuration management (TOML, platform paths)
├── domain/              # Core domain types (Document, Chunk, Embedding, SearchResult, ChunkStrategy)
├── repositories/        # Data persistence layer
│   └── vector_store.rs  # SQLite operations (CRUD, vector search)
├── clients/             # External service adapters
│   └── ollama.rs        # Ollama API client for embeddings
├── services/            # Business logic layer
│   ├── ingestion.rs     # Document ingestion pipeline
│   ├── chunking.rs      # Text chunking strategies
│   └── search.rs        # Semantic search service
└── server/              # Web server (Axum REST API)
```

### Key Design Patterns

1. **Repository Pattern**: `VectorStore` encapsulates all database operations. It manages SQLite connections and provides a clean interface for CRUD operations on Documents, Chunks, and Embeddings.

2. **Service Layer**: Services (`IngestionService`, `SearchService`) orchestrate business logic by composing repositories and clients. They handle workflows like:
   - Ingestion: Load file → Create document → Chunk text → Generate embeddings → Store
   - Search: Generate query embedding → Vector similarity search → Filter by threshold

3. **Domain-Driven Design**: Core domain types (`Document`, `Chunk`, `Embedding`, `SearchResult`) are defined independently of infrastructure concerns in `domain/mod.rs`.

4. **Client Abstraction**: `OllamaClient` abstracts the Ollama API with retry logic, health checks, and batch processing.

### Data Flow

**Ingestion Pipeline**:
```
File → load_file() → Document::new() → chunk_text() →
VectorStore::insert_document() → VectorStore::insert_chunk() →
OllamaClient::embed_batch() → VectorStore::upsert_embedding()
```

**Search Pipeline**:
```
Query → OllamaClient::embed() → VectorStore::search_similar() →
Filter by threshold → Format results (text/json/csv)
```

### Database Schema

SQLite with three main tables:
- **documents**: id, source, content_hash (for deduplication), metadata (JSON), created_at
- **chunks**: id, document_id (FK), chunk_index, content, token_count
- **embeddings**: chunk_id (FK, PK), model, vector (BLOB), dimension

**Important**:
- Embeddings are stored as BLOB (f32 little-endian bytes)
- Vector similarity currently uses naive cosine similarity in Rust (not sqlite-vec extension yet)
- Foreign keys enforced with CASCADE DELETE

## Configuration

Default config at `~/.config/vectdb/config.toml`:
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
strategy = "fixed"

[search]
default_top_k = 10
similarity_threshold = 0.0
```

Override with `--config /path/to/config.toml`

## Dependencies

Key crates:
- **clap**: CLI parsing with derive macros
- **tokio**: Async runtime (required for Ollama HTTP calls and web server)
- **rusqlite**: SQLite bindings with bundled SQLite
- **axum**: Web framework for REST API
- **reqwest**: HTTP client for Ollama API
- **tracing/tracing-subscriber**: Structured logging
- **serde/serde_json**: Serialization
- **sha2**: Content hashing for deduplication

## Testing Patterns

Tests use:
- `VectorStore::in_memory()` for isolated database tests
- `tempfile::NamedTempFile` for file-based tests
- `#[tokio::test]` for async tests
- Tests that require Ollama are designed to gracefully handle unavailability

## Web Server API

Endpoints (default: http://127.0.0.1:3000):
- `GET /` - HTML UI (from static/index.html)
- `GET /api/health` - Health check + Ollama status
- `GET /api/stats` - Database statistics
- `GET /api/search?query=...&top_k=10&threshold=0.0` - Semantic search
- `GET /api/models` - List available Ollama models

**Note**: Each request creates a new `VectorStore` connection (SQLite handles concurrency via WAL mode).

## Important Implementation Details

1. **VectorStore is not thread-safe**: Each command handler creates a new VectorStore instance. The web server creates a new VectorStore per request. This is intentional - rusqlite::Connection is not Send+Sync.

2. **Async/Sync Boundary**: Ollama calls are async (HTTP), database operations are sync (rusqlite). Services like `SearchService` are async but perform sync database operations.

3. **Ollama Retry Logic**: `OllamaClient::embed_with_retry()` implements exponential backoff (3 retries, 100ms initial backoff). 404 errors (model not found) are not retried.

4. **Chunking**: Currently supports FixedSize and Semantic strategies. FixedSize chunks by character count with overlap. Semantic uses unicode-segmentation for sentence boundaries.

5. **Vector Similarity**: Currently uses naive cosine similarity (O(n) scan). Future work will integrate sqlite-vec for efficient HNSW/IVF indexing.

6. **Deduplication**: Documents are deduplicated by SHA-256 hash of content. Ingesting the same file twice will skip the second ingestion.

## Common Development Workflows

### Adding a new command
1. Add variant to `Commands` enum in `src/cli/mod.rs`
2. Add handler function in `src/main.rs` (e.g., `handle_new_command`)
3. Update match statement in `execute_command()`

### Adding a new database operation
1. Add method to `VectorStore` in `src/repositories/vector_store.rs`
2. Add unit test in the same file
3. Use the new method in service layer

### Adding a new API endpoint
1. Add handler function in `src/server/mod.rs`
2. Add route in `serve()` function
3. Define request/response types at bottom of file

## External Dependencies

**Requires Ollama to be running** for embedding generation:
```bash
# Install (macOS)
brew install ollama

# Start service
ollama serve

# Pull embedding model
ollama pull nomic-embed-text
```

Recommended embedding models: nomic-embed-text, all-minilm, mxbai-embed-large

## Roadmap

**Completed phases**:
- ✅ Phase 1: Foundation (CLI structure, config, domain types)
- ✅ Phase 2: Vector Store (SQLite, CRUD operations)
- ✅ Phase 3: Ollama Integration
- ✅ Phase 4: Ingestion Pipeline
- ✅ Phase 5: Search Implementation
- ✅ Phase 6: Web Server & REST API

**Current phase**:
- 🚧 Phase 7: Polish & Documentation
  - [x] Comprehensive documentation suite (architecture, prd, design, process, status, plan)
  - [x] Build scripts (build.sh, serve.sh) with build metadata injection
  - [x] Web UI footer with copyright and build info
  - [x] Live WASM demo on GitHub Pages
  - [ ] Tutorial documentation
  - [ ] Performance benchmarks
  - [ ] Example datasets
  - [ ] Release binaries for major platforms
  - [ ] CI/CD pipeline

**Next priorities**:
- Complete Phase 7 (tutorials, benchmarks, release v0.1.0)
- Set up CI/CD with GitHub Actions
- Integrate sqlite-vec extension for 10-100x faster search
- Add PDF document support
- Implement hybrid search (BM25 + vector)

For detailed roadmap and priorities, see **[documentation/plan.md](documentation/plan.md)**.
