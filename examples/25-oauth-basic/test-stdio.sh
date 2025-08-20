#!/bin/bash

# Test stdio communication with the OAuth server

echo "Testing OAuth MCP Server via STDIO"
echo "==================================="
echo ""

# Send initialize request
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}' | \
    /Users/guy/Development/mcp/sdk/rust-mcp-sdk/target/debug/oauth-basic stdio 2>/dev/null | \
    head -1 | jq .

echo ""
echo "If you see a response above with server info, the server is working correctly."