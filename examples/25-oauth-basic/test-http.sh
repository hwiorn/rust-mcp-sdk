#!/bin/bash

# Test script for OAuth HTTP MCP server using HTTP client example

SERVER_URL="http://localhost:8080"

echo "Testing OAuth MCP HTTP Server at $SERVER_URL"
echo "=========================================="
echo ""
echo "Using the HTTP client example to test the server..."
echo ""

# Run the HTTP client example
cd /Users/guy/Development/mcp/sdk/rust-mcp-sdk

# Build the client if needed
echo "Building HTTP client..."
cargo build --example 24_streamable_http_client --features streamable-http 2>/dev/null

# Run the client to test our OAuth server
echo "Running client tests..."
echo ""
cargo run --example 24_streamable_http_client --features streamable-http 2>/dev/null