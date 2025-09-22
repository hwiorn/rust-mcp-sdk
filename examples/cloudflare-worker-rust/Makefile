# Makefile for Cloudflare Worker MCP Example

.PHONY: help build build-sdk deploy clean test

# Default target
help:
	@echo "Available targets:"
	@echo "  build       - Build the standalone Worker (no SDK)"
	@echo "  build-sdk   - Build the SDK-backed Worker variant"
	@echo "  deploy      - Deploy to Cloudflare Workers"
	@echo "  clean       - Clean build artifacts"
	@echo "  test        - Test the deployed worker"

# Build the standalone Worker (original, no SDK)
build:
	wasm-pack build --target web --out-dir pkg
	@echo "Built standalone worker (no SDK) to pkg/"

# Build the SDK-backed Worker variant
build-sdk:
	wasm-pack build --target web --out-dir pkg-sdk --manifest-path Cargo_sdk.toml
	@echo "Built SDK-backed worker to pkg-sdk/"
	@echo "To use SDK version, update wrangler.toml main to point to pkg-sdk"

# Deploy to Cloudflare Workers
deploy: build
	wrangler deploy
	@echo "Deployed to Cloudflare Workers"

# Clean build artifacts
clean:
	rm -rf pkg pkg-sdk target
	@echo "Cleaned build artifacts"

# Test the deployed worker
test:
	@echo "Testing MCP server..."
	@echo "\nInitializing..."
	@curl -X POST https://mcp-server-sdk.$$(wrangler whoami | grep -o '[^@]*@[^@]*' | cut -d@ -f1).workers.dev \
		-H "Content-Type: application/json" \
		-d '{"jsonrpc":"2.0","id":"1","method":"initialize","params":{"protocolVersion":"2024-11-05","clientInfo":{"name":"test","version":"1.0.0"}}}' \
		2>/dev/null | jq .
	@echo "\nListing tools..."
	@curl -X POST https://mcp-server-sdk.$$(wrangler whoami | grep -o '[^@]*@[^@]*' | cut -d@ -f1).workers.dev \
		-H "Content-Type: application/json" \
		-d '{"jsonrpc":"2.0","id":"2","method":"tools/list","params":{}}' \
		2>/dev/null | jq .