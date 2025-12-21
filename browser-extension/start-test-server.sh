#!/bin/bash
# Simple test server for the browser extension test page

cd "$(dirname "$0")"
PORT=${1:-8080}

echo "Starting test server on port $PORT..."
echo "Open http://localhost:$PORT/test-page.html in your browser"
echo "Press Ctrl+C to stop"
echo ""

python3 -m http.server $PORT 2>/dev/null || python -m SimpleHTTPServer $PORT



