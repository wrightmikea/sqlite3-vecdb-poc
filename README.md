# VectDB - Vector Database CLI

A Rust-based CLI application for semantic search using SQLite's vector extension (sqlite-vec) and local Ollama embedding models.

## Overview

VectDB is a self-contained, local-first vector database that enables semantic search across your document collections. It combines the reliability of SQLite with the power of modern embedding models, all running locally on your machine.

### Key Features

- **Local-First**: No external API dependencies or costs - everything runs on your machine
- **Privacy-Focused**: Your data never leaves your computer
- **Easy to Use**: Simple CLI with intuitive commands
- **Flexible**: Support for multiple embedding models via Ollama
- **Persistent**: All data stored in a portable SQLite database
- **Observable**: Structured logging with configurable verbosity

### Web Interface

VectDB includes a built-in web interface for easy semantic search:

![VectDB Web Interface](images/screenshot.png)

**Try the live demo**: [https://wrightmikea.github.io/sqlite3-vecdb-poc/](https://wrightmikea.github.io/sqlite3-vecdb-poc/)

The live demo runs entirely in your browser using WebAssembly, with pre-loaded example data.

## Documentation

Comprehensive documentation is available in the `documentation/` directory:

- **[Architecture](documentation/architecture.md)** - System architecture, design patterns, and technical decisions
- **[Product Requirements](documentation/prd.md)** - Product vision, features, and user stories
- **[Design](documentation/design.md)** - Detailed component designs, algorithms, and interfaces
- **[Development Process](documentation/process.md)** - TDD workflow, quality checks, and commit guidelines
- **[Project Status](documentation/status.md)** - Current progress, metrics, and known issues
- **[Development Plan](documentation/plan.md)** - Roadmap, next steps, and priorities

For developers working on VectDB, start with [CLAUDE.md](CLAUDE.md) for build commands and development guidelines.

## Current Status: Phase 6 Complete ✅

All core features have been implemented:

- [x] Phase 1: Foundation (CLI structure, config, domain types)
- [x] Phase 2: Vector Store (SQLite, CRUD operations)
- [x] Phase 3: Ollama Integration (embeddings, health checks)
- [x] Phase 4: Ingestion Pipeline (file loading, chunking, deduplication)
- [x] Phase 5: Search Implementation (semantic search, multiple output formats)
- [x] Phase 6: Web Server & REST API (Axum-based HTTP API)

## Installation

### Prerequisites

1. **Rust** (1.70 or later)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Ollama** (for embedding generation in later phases)
   ```bash
   # macOS
   brew install ollama

   # Or download from https://ollama.ai
   ```

### Build from Source

```bash
git clone https://github.com/wrightmikea/sqlite3-vecdb-poc
cd sqlite3-vecdb-poc
./scripts/build.sh
```

The binary will be available at `target/release/vectdb`.

**Note**: The `build.sh` script generates build information (hostname, git commit, timestamp) that is displayed in the web UI footer.

## Quick Start

### 1. Initialize Configuration

```bash
cargo run -- init
```

This creates a configuration file at `~/.config/vectdb/config.toml` with sensible defaults.

### 2. View Available Commands

```bash
cargo run -- --help
```

### 3. Test the CLI

```bash
# All commands currently show placeholder messages
cargo run -- stats
cargo run -- search "test query"
cargo run -- models
```

## Configuration

Configuration is stored in `~/.config/vectdb/config.toml` (or platform-specific config directory).

Default configuration:

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

### Custom Configuration

```bash
# Use a custom config file
cargo run -- --config /path/to/config.toml stats

# Override log level
cargo run -- --log-level debug stats
```

## Architecture

VectDB follows a hexagonal (ports and adapters) architecture with clear separation of concerns:

```
src/
├── cli/           # Command-line interface definitions
├── config/        # Configuration management
├── domain/        # Core domain types and business logic
├── repositories/  # Data persistence (VectorStore)
├── clients/       # External services (OllamaClient)
├── services/      # Business logic (Ingestion, Search)
├── server/        # Web server and REST API
├── error.rs       # Error types and Result alias
└── main.rs        # Application entry point
```

### Core Domain Types

- **Document**: Represents an ingested document with metadata
- **Chunk**: A segment of text from a document
- **Embedding**: Vector representation of a chunk
- **SearchResult**: Result from a semantic search query
- **ChunkStrategy**: Configuration for text chunking

For detailed architecture information, see [documentation/architecture.md](documentation/architecture.md).

## Commands

All commands support `--help` for detailed usage information.

### `init`

Initialize or reset configuration:

```bash
vectdb init [--force]
```

### `ingest`

Ingest documents into the vector database:

```bash
vectdb ingest <PATH> [OPTIONS]

Options:
  -m, --model <MODEL>           Embedding model [default: nomic-embed-text]
  -s, --chunk-size <SIZE>       Chunk size in tokens [default: 512]
  -o, --overlap <SIZE>          Overlap between chunks [default: 50]
  -r, --recursive               Process directories recursively
```

### `search`

Search the vector database:

```bash
vectdb search <QUERY> [OPTIONS]

Options:
  -k, --top-k <K>              Number of results [default: 10]
  -t, --threshold <THRESHOLD>  Similarity threshold [default: 0.0]
  -e, --explain                Show detailed similarity scores
  -f, --format <FORMAT>        Output format: text, json, csv [default: text]
```

### `serve`

Start the web server:

```bash
# Using the serve script
./scripts/serve.sh

# Or with custom port/host
./scripts/serve.sh --port 8080 --host 0.0.0.0

# Or directly
vectdb serve [OPTIONS]

Options:
  -p, --port <PORT>  Server port [default: 3000]
  -H, --host <HOST>  Server host [default: 127.0.0.1]
```

Access the web interface at `http://localhost:3000` (or your configured host/port).

### `stats`

Display database statistics:

```bash
vectdb stats
```

### `optimize`

Optimize database performance (planned):

```bash
vectdb optimize
```

### `models`

List available Ollama models:

```bash
vectdb models
```

## Development

### Running Tests

```bash
cargo test
```

### Running with Logging

```bash
# Info level (default)
cargo run -- stats

# Debug level
cargo run -- --log-level debug stats

# Trace level (very verbose)
RUST_LOG=trace cargo run -- stats
```

### Code Structure

The codebase follows Rust best practices:

- **Module organization**: Clear separation of concerns
- **Error handling**: Custom error types with context
- **Testing**: Unit tests with good coverage
- **Documentation**: Inline documentation and examples
- **Type safety**: Leveraging Rust's type system for correctness

## Roadmap

All core phases (1-6) are complete! Current work focuses on Phase 7 (polish and documentation).

### Next Steps

- Complete comprehensive documentation suite
- Create tutorial documentation
- Performance benchmarking
- Release v0.1.0 with binaries
- Set up CI/CD pipeline
- PDF support
- SQLite-vec integration for improved performance

For detailed roadmap and priorities, see [documentation/plan.md](documentation/plan.md).

## Technology Stack

- **Rust 2024**: Core language
- **Clap 4.x**: CLI parsing with derive macros
- **Tokio**: Async runtime
- **Tracing**: Structured logging
- **Serde**: Serialization (JSON, TOML)
- **SQLite + rusqlite**: Vector database storage
- **Reqwest**: HTTP client for Ollama API
- **Axum**: Web framework for REST API
- **wasm-bindgen**: WebAssembly bindings for demo

## Examples

Try VectDB with the included example documents:

```bash
# Ingest example documents
cargo run -- ingest examples/documents/ -r

# Search the examples
cargo run -- search "What is Rust?" -k 3
cargo run -- search "vector database similarity" --explain
```

See `examples/README.md` for more details.

## Contributing

Contributions are welcome! Please feel free to submit issues, feature requests, or pull requests.

## License

MIT License - Copyright (c) 2025 Michael A. Wright

See [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built following the architecture outlined in `docs/vector-db-research.md`
- Inspired by modern vector database systems like Qdrant and Weaviate
- Leverages the excellent Rust ecosystem

---

**Status**: Phase 6 Complete - Full functionality implemented
**License**: MIT
**Last Updated**: 2025-11-05
