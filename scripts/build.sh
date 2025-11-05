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
echo ""
echo "To run the server:"
echo "   ./scripts/serve.sh"
echo "   or"
echo "   ./target/release/vectdb serve"
