#!/bin/bash
set -e

# VectDB Serve Script
# Starts the VectDB web server

# Default values
HOST="${VECTDB_HOST:-127.0.0.1}"
PORT="${VECTDB_PORT:-3000}"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--port)
            PORT="$2"
            shift 2
            ;;
        -h|--host)
            HOST="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -p, --port PORT    Server port (default: 3000)"
            echo "  -h, --host HOST    Server host (default: 127.0.0.1)"
            echo "  --help             Show this help message"
            echo ""
            echo "Environment variables:"
            echo "  VECTDB_HOST        Server host (overridden by -h/--host)"
            echo "  VECTDB_PORT        Server port (overridden by -p/--port)"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Run '$0 --help' for usage information"
            exit 1
            ;;
    esac
done

# Check if binary exists
BINARY="./target/release/vectdb"
if [ ! -f "$BINARY" ]; then
    echo "❌ Binary not found: $BINARY"
    echo ""
    echo "Please build the project first:"
    echo "   ./scripts/build.sh"
    exit 1
fi

echo "🚀 Starting VectDB server..."
echo "   Host: $HOST"
echo "   Port: $PORT"
echo ""
echo "Web UI:  http://$HOST:$PORT"
echo "API:     http://$HOST:$PORT/api"
echo ""
echo "Press Ctrl+C to stop"
echo ""

# Run the server
exec "$BINARY" serve --host "$HOST" --port "$PORT"
