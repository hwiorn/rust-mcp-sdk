#!/bin/bash
# Silent runner for MCP OAuth example - suppresses all cargo output

# Build first (output to stderr)
cargo build --package oauth-basic 2>/dev/null

# Run the built binary directly (no cargo output)
exec ./../../target/debug/oauth-basic stdio