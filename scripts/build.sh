#!/bin/bash
set -e

# VectDB Build Script
# Builds the project and generates build information

echo "🔨 Building VectDB..."

# Get build information
BUILD_HOST=$(hostname)
BUILD_COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
BUILD_TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

echo "📋 Build Information:"
echo "   Host:      $BUILD_HOST"
echo "   Commit:    $BUILD_COMMIT"
echo "   Timestamp: $BUILD_TIMESTAMP"

# Generate build-info.js
echo ""
echo "📝 Generating build-info.js..."
cat > static/build-info.js <<EOF
// Auto-generated build information
// Generated at: $BUILD_TIMESTAMP
window.BUILD_INFO = {
  host: "$BUILD_HOST",
  commit: "$BUILD_COMMIT",
  timestamp: "$BUILD_TIMESTAMP"
};
EOF

echo "✅ Build info generated"

# Build the project
echo ""
echo "🦀 Running cargo build --release..."
cargo build --release

echo ""
echo "✅ Build complete!"
echo "   Binary: target/release/vectdb"

# Build WASM demo
echo ""
echo "🌐 Building WASM demo..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "⚠️  wasm-pack not found. Skipping WASM demo build."
    echo "   Install with: cargo install wasm-pack"
else
    cd demo
    wasm-pack build --target web --out-dir ../docs --release
    cd ..

    # Copy the HTML file and favicon
    cp demo/index.html docs/
    cp static/favicon.ico docs/

    # Generate build-info.js for demo
    cat > docs/build-info.js <<EOF
// Auto-generated build information for demo
// Generated at: $BUILD_TIMESTAMP
window.BUILD_INFO = {
  host: "$BUILD_HOST",
  commit: "$BUILD_COMMIT",
  timestamp: "$BUILD_TIMESTAMP"
};
EOF

    echo "✅ WASM demo built and copied to docs/"
fi

echo ""
echo "To run the server:"
echo "   ./scripts/serve.sh"
echo "   or"
echo "   ./target/release/vectdb serve"
echo ""
echo "Demo available at: docs/index.html"
echo "GitHub Pages will serve from: https://wrightmikea.github.io/sqlite3-vecdb-poc/"
