# Makefile for Cloudflare Worker MCP Example
# Provides consistent commands for building, testing, and deploying

.PHONY: help build build-debug dev deploy clean test test-local test-prod logs check

# Configuration
WORKER_NAME = mcp-sdk-worker
PROD_URL = https://mcp-sdk-worker.guy-ernest.workers.dev
LOCAL_URL = http://localhost:8787
WASM_TARGET = wasm32-unknown-unknown

# Default target
help:
	@echo "╔══════════════════════════════════════════════════════════╗"
	@echo "║     Cloudflare Worker MCP Server - Build & Deploy        ║"
	@echo "╚══════════════════════════════════════════════════════════╝"
	@echo ""
	@echo "🚀 Quick Start:"
	@echo "  make build          - Build WASM module"
	@echo "  make dev            - Start local dev server"
	@echo "  make deploy         - Deploy to Cloudflare"
	@echo "  make test-prod      - Test production deployment"
	@echo ""
	@echo "🛠️  Build Commands:"
	@echo "  make build          - Production build (optimized)"
	@echo "  make build-debug    - Debug build (readable)"
	@echo "  make clean          - Remove build artifacts"
	@echo ""
	@echo "🧪 Testing:"
	@echo "  make test-local     - Test local instance"
	@echo "  make test-prod      - Test production"
	@echo "  make logs           - View production logs"

# Build WASM module for production
build:
	@echo "Building WASM module..."
	@wasm-pack build --target web --out-dir pkg --no-opt
	@echo "✅ Built to pkg/ (Size: $$(du -sh pkg/*.wasm | cut -f1))"

# Build debug version
build-debug:
	@echo "Building debug WASM module..."
	@wasm-pack build --dev --target web --out-dir pkg
	@echo "✅ Built debug version to pkg/"

# Start local dev server
dev: build
	@echo "Starting dev server on $(LOCAL_URL)..."
	@wrangler dev

# Deploy to production
deploy: build
	@echo "Deploying to Cloudflare..."
	@wrangler deploy
	@echo "✅ Deployed to $(PROD_URL)"

# Clean build artifacts
clean:
	@rm -rf pkg pkg-sdk target build
	@echo "✅ Cleaned build artifacts"

# Run all tests
test: test-prod

# Test local instance
test-local:
	@echo "Testing local server..."
	@curl -s -X POST $(LOCAL_URL) \
		-H "Content-Type: application/json" \
		-d '{"jsonrpc":"2.0","id":"1","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}' \
		2>/dev/null | jq '.result.serverInfo' || echo "❌ Server not running"

# Test production
test-prod:
	@echo "Testing production server..."
	@echo "1. Initialize:"
	@curl -s -X POST $(PROD_URL) \
		-H "Content-Type: application/json" \
		-d '{"jsonrpc":"2.0","id":"1","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}' \
		| jq '.result.serverInfo'
	@echo "\n2. List tools:"
	@curl -s -X POST $(PROD_URL) \
		-H "Content-Type: application/json" \
		-d '{"jsonrpc":"2.0","id":"2","method":"tools/list","params":{}}' \
		| jq '.result.tools[].name'
	@echo "\n3. Call tool:"
	@curl -s -X POST $(PROD_URL) \
		-H "Content-Type: application/json" \
		-d '{"jsonrpc":"2.0","id":"3","method":"tools/call","params":{"name":"calculator","arguments":{"operation":"add","a":5,"b":3}}}' \
		| jq '.result.content[0].text' -r | jq '.'

# View production logs
logs:
	@wrangler tail $(WORKER_NAME)

# Check prerequisites
check:
	@command -v rustc >/dev/null && echo "✓ Rust installed" || echo "✗ Rust missing"
	@rustup target list --installed | grep -q $(WASM_TARGET) && echo "✓ WASM target" || echo "✗ WASM target missing"
	@command -v wasm-pack >/dev/null && echo "✓ wasm-pack" || echo "✗ wasm-pack missing"
	@command -v wrangler >/dev/null && echo "✓ wrangler" || echo "✗ wrangler missing"