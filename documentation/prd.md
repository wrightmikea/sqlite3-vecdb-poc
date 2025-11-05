# Product Requirements Document (PRD)

## VectDB - Local-First Vector Database

**Version**: 1.0
**Last Updated**: 2025-11-05
**Status**: Phase 6 Complete

## Executive Summary

VectDB is a local-first vector database that enables semantic search across document collections without requiring external API dependencies or cloud services. It combines SQLite's reliability with modern embedding models (via Ollama) to provide privacy-focused semantic search capabilities.

## Problem Statement

### The Need

Developers and researchers need to perform semantic search over their own documents but face several challenges:

1. **Privacy Concerns**: Cloud-based vector databases send data to external servers
2. **API Costs**: Commercial embedding services charge per query/token
3. **Complexity**: Setting up production vector databases requires infrastructure expertise
4. **Vendor Lock-in**: Proprietary solutions create dependencies on specific providers

### Current Solutions

- **Cloud Vector DBs** (Pinecone, Weaviate): Require internet, have costs, privacy concerns
- **Local Embedding Models**: Exist but lack integrated storage and search infrastructure
- **Full-Stack Solutions** (Chroma, LanceDB): Still require Python/Node runtime, can be heavy

## Target Audience

### Primary Users

1. **Developers**: Building local-first AI applications
2. **Researchers**: Processing academic papers and research notes privately
3. **Enterprise Users**: Organizations with strict data privacy requirements
4. **Students**: Learning about vector databases and semantic search

### User Personas

**Alex - Privacy-Conscious Developer**
- Wants to build AI features without sending data to cloud services
- Needs lightweight, embeddable solution
- Values Rust's performance and safety guarantees

**Dr. Chen - Academic Researcher**
- Has large collection of PDFs and notes
- Needs semantic search across documents
- Limited technical infrastructure support

**Sarah - Enterprise Engineer**
- Must comply with data residency regulations
- Needs audit trail and local deployment
- Requires production-ready reliability

## Goals and Objectives

### Primary Goals

1. **Privacy**: All data processing happens locally, no external API calls except local Ollama
2. **Simplicity**: Single binary, minimal dependencies, easy setup
3. **Performance**: Fast ingestion and sub-second search queries
4. **Reliability**: SQLite-backed storage with ACID guarantees

### Success Metrics

- **Time to First Search**: < 5 minutes from installation
- **Ingestion Speed**: > 100 documents/minute (moderate size)
- **Search Latency**: < 1 second for typical queries (< 10k chunks)
- **Binary Size**: < 50MB (release build)
- **Memory Usage**: < 500MB for typical workloads

## Features and Requirements

### Phase 1: Foundation ✅

**Status**: Complete

- [x] Project structure and module organization
- [x] Core domain types (Document, Chunk, Embedding, SearchResult)
- [x] Configuration management (TOML, platform-specific paths)
- [x] CLI skeleton with clap
- [x] Error handling infrastructure
- [x] Structured logging

### Phase 2: Vector Store ✅

**Status**: Complete

- [x] SQLite database with schema
- [x] CRUD operations for documents, chunks, embeddings
- [x] Vector similarity search (cosine similarity)
- [x] Content deduplication (SHA-256 hashing)
- [x] Foreign key constraints and cascading deletes
- [x] WAL mode for concurrency

### Phase 3: Ollama Integration ✅

**Status**: Complete

- [x] HTTP client for Ollama API
- [x] Embedding generation with retry logic
- [x] Batch processing
- [x] Health checks
- [x] Model listing
- [x] Configurable timeout and retry policy

### Phase 4: Ingestion Pipeline ✅

**Status**: Complete

- [x] File loading (text, markdown)
- [x] Document creation with metadata
- [x] Text chunking strategies:
  - Fixed-size with overlap
  - Semantic (sentence/paragraph boundaries)
- [x] Embedding generation pipeline
- [x] Progress reporting
- [x] Recursive directory processing
- [x] Error handling and recovery

### Phase 5: Search Implementation ✅

**Status**: Complete

- [x] Query embedding generation
- [x] Vector similarity search
- [x] Threshold filtering
- [x] Top-K results
- [x] Multiple output formats (text, JSON, CSV)
- [x] Similarity score display
- [x] Source attribution

### Phase 6: Web Server & REST API ✅

**Status**: Complete

- [x] Axum-based HTTP server
- [x] REST API endpoints:
  - GET / (HTML UI)
  - GET /api/health (service status)
  - GET /api/stats (database statistics)
  - GET /api/search (semantic search)
  - GET /api/models (list Ollama models)
- [x] Static file serving
- [x] Web UI with search interface
- [x] Build information display
- [x] WebAssembly live demo

### Phase 7: Polish & Documentation (Planned)

**Status**: In Progress

- [ ] Performance benchmarks
- [ ] Example datasets
- [ ] Tutorial documentation
- [ ] Release binaries for major platforms
- [ ] CI/CD pipeline
- [ ] Homebrew/Cargo installation
- [ ] Video tutorials

## Non-Functional Requirements

### Performance

- **Ingestion**: 100+ documents/minute
- **Search**: < 1 second for typical queries
- **Startup**: < 100ms cold start
- **Memory**: < 500MB typical usage

### Scalability

- **Documents**: Tested up to 10,000 documents
- **Chunks**: Support for 100,000+ chunks
- **Embeddings**: Millions of vectors (limited by SQLite)

### Security

- **No External Calls**: All processing local (except Ollama localhost)
- **Content Hashing**: SHA-256 for deduplication
- **Input Validation**: All user inputs sanitized
- **No Secrets**: No API keys or credentials required

### Reliability

- **ACID Transactions**: SQLite guarantees
- **Crash Recovery**: WAL mode for durability
- **Error Handling**: Graceful degradation
- **Data Integrity**: Foreign key constraints

### Usability

- **Single Binary**: No runtime dependencies
- **Self-Documenting**: Built-in help and examples
- **Sensible Defaults**: Works out of box
- **Progressive Disclosure**: Advanced features optional

## User Stories

### Core Workflows

**Story 1: First-Time Setup**
```
AS A new user
I WANT to install and run VectDB quickly
SO THAT I can start searching my documents

Acceptance Criteria:
- Installation takes < 5 minutes
- Default configuration works without editing
- First search completes successfully
```

**Story 2: Document Ingestion**
```
AS A user with a folder of documents
I WANT to ingest all my text files recursively
SO THAT I can search across my entire collection

Acceptance Criteria:
- Single command ingests entire directory
- Progress is visible
- Duplicates are skipped automatically
- Errors don't stop processing
```

**Story 3: Semantic Search**
```
AS A user looking for information
I WANT to search using natural language queries
SO THAT I can find relevant content without exact keywords

Acceptance Criteria:
- Query returns semantically similar results
- Results show similarity scores
- Source documents are clearly attributed
- Results are ranked by relevance
```

**Story 4: Web Interface**
```
AS A user who prefers GUIs
I WANT to access VectDB through a web browser
SO THAT I don't need to use command line

Acceptance Criteria:
- UI is intuitive and responsive
- Search results are well-formatted
- Statistics are visible
- Works on localhost without configuration
```

## Technical Requirements

### Dependencies

**Required**:
- Rust 2024 toolchain
- Ollama (for embedding generation)

**Optional**:
- wasm-pack (for building demo)
- npx/http-server (for local demo testing)

### Platform Support

- **macOS**: Primary development platform
- **Linux**: Full support
- **Windows**: Full support (via WSL or native)

### Integration Points

1. **Ollama API** (localhost:11434)
   - Embedding generation
   - Model management

2. **File System**
   - Document reading
   - Configuration storage
   - Database persistence

3. **HTTP Server** (localhost:3000)
   - REST API
   - Web UI serving

## Future Enhancements

### Short Term

1. **PDF Support**: Ingest PDF documents
2. **More Chunking Strategies**: Code-aware, markdown-aware
3. **Hybrid Search**: Combine vector + keyword search
4. **Batch Operations**: Bulk delete, re-index

### Medium Term

1. **SQLite-vec Integration**: Efficient vector indexing (HNSW/IVF)
2. **Multiple Models**: Support different embeddings per corpus
3. **Document Versions**: Track changes over time
4. **Export/Import**: Database backup and migration

### Long Term

1. **Distributed Mode**: Optional multi-node support
2. **Plugin System**: Custom embedding providers
3. **Advanced Analytics**: Usage patterns, performance metrics
4. **RAG Integration**: Built-in retrieval-augmented generation

## Out of Scope

The following features are explicitly **not** planned:

- Cloud deployment or SaaS offering
- Built-in LLM integration (use with external LLMs)
- Real-time collaborative features
- Mobile applications
- GUI application (web UI is sufficient)
- Commercial support

## Success Criteria

### Launch Criteria (v1.0)

- [ ] All Phase 1-6 features complete and tested
- [ ] Documentation complete (README, architecture, tutorials)
- [ ] Release binaries available
- [ ] Live demo accessible
- [ ] At least 10 GitHub stars
- [ ] Zero critical bugs

### Long-Term Success

- 100+ GitHub stars
- Active community contributions
- Featured in Rust or vector database communities
- Used in production by at least 5 organizations
- Cited in academic papers or tutorials

## Open Questions

1. **Embedding Model Strategy**: Should we bundle specific models or always defer to Ollama?
2. **SQLite-vec Timeline**: When to integrate the extension?
3. **Multi-language Support**: Priority for internationalization?
4. **Commercial Use**: Any restrictions on usage?

## References

- [Ollama Documentation](https://github.com/ollama/ollama)
- [SQLite Vector Search Extensions](https://github.com/asg017/sqlite-vss)
- [Semantic Search Best Practices](https://www.pinecone.io/learn/semantic-search/)
