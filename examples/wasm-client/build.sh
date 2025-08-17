#!/bin/bash
set -e

# This script builds the WASM client and prepares it for use.

# 1. Build the WASM package
export CARGO_PROFILE_RELEASE_LTO=false
wasm-pack build --target web --out-name mcp_wasm_client --no-opt

# 2. Copy the generated files to the example root
cp pkg/mcp_wasm_client_bg.wasm .
cp pkg/mcp_wasm_client.js .

echo "Build complete. Serve this directory with a web server."
