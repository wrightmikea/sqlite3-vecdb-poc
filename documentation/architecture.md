# VectDB Architecture

## Overview

VectDB follows a **hexagonal (ports and adapters) architecture** with clear separation of concerns. This document describes the architectural decisions, patterns, and design principles used in the project.

## Architecture Pattern

### Hexagonal Architecture (Ports & Adapters)

The codebase is organized around the hexagonal architecture pattern to achieve:
- **Independence from frameworks**: Core domain logic is isolated from infrastructure
- **Testability**: Business logic can be tested without external dependencies
- **Flexibility**: Easy to swap implementations (e.g., different vector stores or embedding providers)

```
┌─────────────────────────────────────────────────────────────┐
│                        CLI / HTTP API                        │
│                      (Adapter - Entry)                       │
└────────────────────────┬────────────────────────────────────┘
                         │
┌────────────────────────┴────────────────────────────────────┐
│                     Services (Use Cases)                     │
│          IngestionService, SearchService                     │
└────┬─────────────────────────────────────────────────┬──────┘
     │                                                   │
     │ ┌──────────────────────────────────────────┐    │
     │ │         Domain (Core Business Logic)      │    │
     │ │   Document, Chunk, Embedding, Strategy   │    │
     │ └──────────────────────────────────────────┘    │
     │                                                   │
┌────┴──────────────────┐                  ┌───────────┴──────┐
│  Repositories (Port)   │                  │  Clients (Port)  │
│   VectorStore          │                  │   OllamaClient   │
└────────┬───────────────┘                  └────────┬─────────┘
         │                                           │
┌────────┴───────────────┐           ┌──────────────┴─────────┐
│ SQLite (Adapter)       │           │ HTTP/Ollama (Adapter)  │
└────────────────────────┘           └────────────────────────┘
```

## Module Structure

### Core Modules

#### `domain/`
**Purpose**: Core business entities and value objects

Contains:
- `Document`: Represents ingested documents with metadata and content hashing
- `Chunk`: Text segments extracted from documents
- `Embedding`: Vector representations of chunks
- `SearchResult`: Query results with similarity scores
- `ChunkStrategy`: Configuration for text segmentation strategies

**Dependencies**: None (pure Rust, minimal external dependencies)

#### `repositories/`
**Purpose**: Data persistence abstraction (Port)

- `VectorStore`: SQLite-based vector storage implementation
  - CRUD operations for documents, chunks, embeddings
  - Vector similarity search (currently cosine similarity)
  - Database schema management
  - WAL mode for concurrency

**Key Design Decisions**:
- Not thread-safe by design (rusqlite::Connection is !Send+!Sync)
- Each operation creates a new connection (SQLite handles concurrency via WAL)
- Embeddings stored as BLOB (f32 little-endian bytes)
- Content deduplication via SHA-256 hashing

#### `clients/`
**Purpose**: External service adapters (Port)

- `OllamaClient`: HTTP client for Ollama embedding API
  - Exponential backoff retry logic
  - Health checks
  - Batch processing
  - Model listing

**Key Design Decisions**:
- Async interface (uses tokio)
- Graceful degradation (returns Ok(false) instead of errors for unavailable service)
- 404 errors (model not found) are not retried

#### `services/`
**Purpose**: Business logic orchestration (Use Cases)

- `IngestionService`: Orchestrates document ingestion workflow
  - File loading → Document creation → Chunking → Embedding generation → Storage
  - Handles deduplication
  - Progress reporting

- `SearchService`: Semantic search implementation
  - Query embedding generation
  - Vector similarity search
  - Result formatting (text, JSON, CSV)
  - Threshold filtering

- `chunking`: Text segmentation strategies
  - Fixed-size with overlap
  - Semantic (sentence/paragraph boundaries)

**Key Design Decisions**:
- Services are async (coordinate between sync SQLite and async HTTP)
- Services compose repositories and clients
- Each service handles a specific use case

#### `server/`
**Purpose**: HTTP API adapter (Axum web server)

- REST API endpoints
- Static file serving
- State management

**Key Design Decisions**:
- Each request creates a new VectorStore instance
- Shared OllamaClient (wrapped in Arc)
- Tower middleware for tracing

### Supporting Modules

#### `cli/`
Command-line interface definitions using clap derive macros

#### `config/`
Configuration management (TOML files, platform-specific paths)

#### `error.rs`
Unified error type (VectDbError) and Result alias

## Data Flow

### Ingestion Pipeline

```
1. User/CLI
   ↓ (file path)
2. IngestionService::ingest_file()
   ↓ (read file, compute SHA-256)
3. VectorStore::find_by_hash()
   ↓ (check deduplication)
4. [if new] VectorStore::insert_document()
   ↓ (document_id)
5. chunking::chunk_text()
   ↓ (chunks)
6. VectorStore::insert_chunk() × N
   ↓ (chunk_ids)
7. OllamaClient::embed_batch()
   ↓ (embeddings)
8. VectorStore::upsert_embedding() × N
   ↓
9. Return IngestionResult
```

### Search Pipeline

```
1. User/CLI/API
   ↓ (query string)
2. SearchService::search()
   ↓
3. OllamaClient::embed()
   ↓ (query embedding)
4. VectorStore::search_similar()
   ↓ (all embeddings + chunks + documents)
5. [in-memory] cosine_similarity()
   ↓ (scored results)
6. Filter by threshold
   ↓ (filtered results)
7. Sort by similarity (descending)
   ↓ (top-k results)
8. format_results_*()
   ↓
9. Return formatted output
```

## Database Schema

### Tables

**documents**
- `id`: INTEGER PRIMARY KEY AUTOINCREMENT
- `source`: TEXT NOT NULL
- `content_hash`: TEXT UNIQUE NOT NULL (SHA-256)
- `metadata`: TEXT (JSON)
- `created_at`: INTEGER NOT NULL (Unix timestamp)

**chunks**
- `id`: INTEGER PRIMARY KEY AUTOINCREMENT
- `document_id`: INTEGER NOT NULL (FK → documents.id ON DELETE CASCADE)
- `chunk_index`: INTEGER NOT NULL
- `content`: TEXT NOT NULL
- `token_count`: INTEGER
- UNIQUE(document_id, chunk_index)

**embeddings**
- `chunk_id`: INTEGER PRIMARY KEY (FK → chunks.id ON DELETE CASCADE)
- `model`: TEXT NOT NULL
- `vector`: BLOB NOT NULL (f32 array, little-endian)
- `dimension`: INTEGER NOT NULL

### Indices

- `idx_chunks_document`: ON chunks(document_id)
- `idx_embeddings_model`: ON embeddings(model)

## Concurrency Model

### SQLite Concurrency
- WAL (Write-Ahead Logging) mode enabled
- Multiple readers, single writer
- Each VectorStore creates a new connection
- Connection pool not needed (SQLite handles locking)

### Async/Sync Boundary
- **Async**: HTTP calls (Ollama), web server, CLI orchestration
- **Sync**: SQLite operations (rusqlite is sync-only)
- Services bridge the boundary using tokio::task::spawn_blocking (if needed)

## Key Design Decisions

### 1. No Connection Pooling
**Rationale**: SQLite with WAL mode handles concurrent access efficiently. rusqlite::Connection is !Send, making pooling complex. Creating connections is cheap.

### 2. In-Memory Vector Similarity
**Rationale**: Current implementation loads all embeddings and computes cosine similarity in Rust. This is O(n) but simple and sufficient for moderate-scale use cases.

**Future**: Integrate sqlite-vec extension for HNSW/IVF indexing when scaling is needed.

### 3. Embeddings as BLOBs
**Rationale**: Direct f32 array storage is compact and fast. No JSON overhead. Easy to read/write with from_le_bytes/to_le_bytes.

### 4. Content Deduplication
**Rationale**: SHA-256 hashing prevents duplicate ingestion. Hash computed once at ingestion time. Stored in database for efficient lookups.

### 5. Stateless Web Server
**Rationale**: Each HTTP request creates a new VectorStore. No shared mutable state (except Arc<OllamaClient>). Simplifies deployment and scaling.

## Technology Choices

### Core Technologies
- **Rust 2024**: Memory safety, performance, excellent error handling
- **SQLite**: Embedded, zero-config, ACID transactions, cross-platform
- **Tokio**: Async runtime for HTTP and concurrent operations
- **Axum**: Modern, ergonomic web framework
- **wasm-bindgen**: WebAssembly bindings for browser demo

### Key Libraries
- **rusqlite**: SQLite bindings (bundled, no external deps)
- **reqwest**: HTTP client with async support
- **clap**: CLI parsing with derive macros
- **serde**: Serialization (JSON, TOML)
- **tracing**: Structured logging
- **unicode-segmentation**: Unicode-aware text processing

## Testing Strategy

### Unit Tests
- Domain logic (chunking strategies, similarity calculations)
- Repository operations (in-memory SQLite)
- Serialization/deserialization

### Integration Tests
- End-to-end CLI workflows
- HTTP API endpoints
- Ollama integration (with graceful degradation)

### UI Testing
- MCP/Playwright for web interface validation
- Screenshot capture for documentation

## Future Considerations

### Scalability
- Integrate sqlite-vec for efficient vector indexing (HNSW/IVF)
- Batch processing improvements
- Streaming responses for large result sets

### Features
- PDF ingestion
- More chunking strategies (markdown-aware, code-aware)
- Multi-model support (different embeddings for different corpora)
- Hybrid search (vector + keyword)

### Architecture Evolution
- Consider trait-based repository abstraction for multiple backends
- Plugin system for custom embedding providers
- Distributed deployment support (if needed)

## References

- [Hexagonal Architecture](https://alistair.cockburn.us/hexagonal-architecture/)
- [SQLite WAL Mode](https://sqlite.org/wal.html)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
