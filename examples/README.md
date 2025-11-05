# VectDB Examples

This directory contains example documents for testing and demonstrating VectDB functionality.

## Quick Start

Ingest the example documents:

```bash
# From the repository root
cargo run -- ingest examples/documents/ -r

# Or using the compiled binary
vectdb ingest examples/documents/ -r
```

## Search Examples

After ingesting, try these semantic searches:

```bash
# Find information about Rust
vectdb search "What is Rust programming language?"

# Learn about vector databases
vectdb search "How do vector databases work?" -k 5

# Machine learning concepts
vectdb search "supervised learning examples" --explain

# Format as JSON
vectdb search "embeddings and similarity" -f json
```

## Example Documents

The `documents/` directory contains three sample text files:

- **rust_programming.txt**: Introduction to Rust programming language, features, and use cases
- **vector_databases.md**: Overview of vector databases, how they work, and applications
- **machine_learning_basics.txt**: Fundamentals of machine learning, types, algorithms, and concepts

These documents are provided as toy examples for testing VectDB. They cover topics relevant to the project and demonstrate semantic search capabilities.

## Adding Your Own Documents

To ingest your own documents:

1. Create a directory outside this repository (to avoid accidental commits)
2. Add your text files (.txt, .md)
3. Ingest them: `vectdb ingest /path/to/your/documents -r`

**Note**: The repository's `.gitignore` excludes common document directories (`texts/`, `documents/`, `data/`, `corpus/`) and PDF files to prevent accidental publication of copyrighted material.
