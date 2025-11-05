# VectDB Live Demo (WASM)

This directory contains a WebAssembly-based live demo of VectDB that runs entirely in the browser.

## Overview

The demo provides a static, canned demonstration of VectDB's semantic search capabilities without requiring:
- Backend server
- Database
- Ollama installation

It uses pre-loaded example data to show how VectDB works.

## Building

The demo is automatically built when you run:

```bash
./scripts/build.sh
```

This script:
1. Compiles the Rust code to WebAssembly using `wasm-pack`
2. Generates the necessary JavaScript bindings
3. Copies all files to the `docs/` directory
4. Creates a `build-info.js` with build metadata

### Prerequisites

Install `wasm-pack` if you haven't already:

```bash
cargo install wasm-pack
```

## Structure

- **Cargo.toml** - WASM crate configuration
- **src/lib.rs** - Rust code with canned demo data and search logic
- **index.html** - Demo HTML page (similar to the main web UI)

## Demo Data

The demo includes canned search results from the example documents:
- `examples/documents/rust_programming.txt`
- `examples/documents/vector_databases.md`
- `examples/documents/machine_learning_basics.txt`

The search always returns the same pre-defined results to demonstrate the UI and functionality.

## Deployment

The built demo is automatically placed in the `docs/` directory, which is configured for GitHub Pages.

**Live Demo**: https://wrightmikea.github.io/sqlite3-vecdb-poc/

## Local Testing

After building, you can test the demo locally:

```bash
# Using Python
python3 -m http.server 8000 -d docs

# Or using Node.js
npx http-server docs -p 8000
```

Then visit: http://localhost:8000

**Note**: The demo requires a web server due to WASM CORS requirements. Simply opening `index.html` in a browser won't work.
