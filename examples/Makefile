# Makefile for MCP SDK WASM Examples

.PHONY: help build-cloudflare build-spin test-cloudflare test-spin deploy-cloudflare deploy-spin clean

# Default endpoint URLs
CLOUDFLARE_LOCAL_URL ?= http://localhost:8787
SPIN_LOCAL_URL ?= http://localhost:3000
CLOUDFLARE_PROD_URL ?= https://mcp-server-sdk.$(shell wrangler whoami 2>/dev/null | grep -o '[^@]*' | head -1).workers.dev

help:
	@echo "MCP SDK WASM Examples Makefile"
	@echo ""
	@echo "Build targets:"
	@echo "  build-cloudflare    - Build Cloudflare Workers example"
	@echo "  build-spin          - Build Fermyon Spin example"
	@echo "  build-all           - Build both examples"
	@echo ""
	@echo "Test targets:"
	@echo "  test-cloudflare     - Test Cloudflare example (requires wrangler dev running)"
	@echo "  test-spin           - Test Spin example (requires spin up running)"
	@echo ""
	@echo "Deploy targets:"
	@echo "  deploy-cloudflare   - Deploy to Cloudflare Workers"
	@echo "  deploy-spin         - Deploy to Fermyon Cloud"
	@echo ""
	@echo "Development:"
	@echo "  dev-cloudflare      - Start Cloudflare dev server"
	@echo "  dev-spin            - Start Spin dev server"
	@echo ""
	@echo "Utilities:"
	@echo "  clean               - Clean all build artifacts"
	@echo "  install-deps        - Install required dependencies"

# Install dependencies
install-deps:
	@echo "Installing WASM targets..."
	rustup target add wasm32-unknown-unknown
	rustup target add wasm32-wasip1
	@echo "Checking for wasm-pack..."
	@which wasm-pack > /dev/null || (echo "Installing wasm-pack..." && curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh)
	@echo "Checking for wrangler..."
	@which wrangler > /dev/null || (echo "Please install wrangler: npm install -g wrangler" && exit 1)
	@echo "Checking for spin..."
	@which spin > /dev/null || (echo "Please install spin: curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash" && exit 1)
	@echo "Dependencies installed!"

# Build targets
build-cloudflare:
	@echo "Building Cloudflare Workers example..."
	cd cloudflare-worker-rust && cargo build --target wasm32-unknown-unknown --release
	@echo "✅ Cloudflare example built successfully"

build-spin:
	@echo "Building Fermyon Spin example..."
	cd fermyon-spin-rust && cargo build --target wasm32-wasip1 --release
	@echo "✅ Spin example built successfully"

build-all: build-cloudflare build-spin
	@echo "✅ All examples built successfully"

# Development servers
dev-cloudflare: build-cloudflare
	@echo "Starting Cloudflare dev server on port 8787..."
	cd cloudflare-worker-rust && wrangler dev

dev-spin: build-spin
	@echo "Starting Spin dev server on port 3000..."
	cd fermyon-spin-rust && spin up

# Test functions
define test_mcp
	@echo "\n📋 Testing MCP server at: $(1)"
	@echo "\n1️⃣  Initializing..."
	@curl -s -X POST $(1) \
		-H "Content-Type: application/json" \
		-d '{"jsonrpc":"2.0","id":"1","method":"initialize","params":{"protocolVersion":"2024-11-05","clientInfo":{"name":"test","version":"1.0.0"}}}' \
		| jq . || echo "❌ Failed to initialize"
	@echo "\n2️⃣  Listing tools..."
	@curl -s -X POST $(1) \
		-H "Content-Type: application/json" \
		-d '{"jsonrpc":"2.0","id":"2","method":"tools/list","params":{}}' \
		| jq . || echo "❌ Failed to list tools"
	@echo "\n3️⃣  Testing tool call..."
	@curl -s -X POST $(1) \
		-H "Content-Type: application/json" \
		-d '{"jsonrpc":"2.0","id":"3","method":"tools/call","params":{"name":"$(2)","arguments":$(3)}}' \
		| jq . || echo "❌ Failed to call tool"
endef

# Test Cloudflare
test-cloudflare:
	@echo "🧪 Testing Cloudflare Worker..."
	$(call test_mcp,$(CLOUDFLARE_LOCAL_URL),"calculator",'{"operation":"add","a":10,"b":20}')
	@echo "\n✅ Cloudflare test complete"

# Test Spin
test-spin:
	@echo "🧪 Testing Fermyon Spin..."
	$(call test_mcp,$(SPIN_LOCAL_URL),"add",'{"a":15,"b":25}')
	@echo "\n✅ Spin test complete"

# Deploy targets
deploy-cloudflare: build-cloudflare
	@echo "🚀 Deploying to Cloudflare Workers..."
	cd cloudflare-worker-rust && wrangler deploy
	@echo "✅ Deployed! Testing production endpoint..."
	$(call test_mcp,$(CLOUDFLARE_PROD_URL),"calculator",'{"operation":"add","a":5,"b":10}')

deploy-spin: build-spin
	@echo "🚀 Deploying to Fermyon Cloud..."
	cd fermyon-spin-rust && spin deploy
	@echo "✅ Deployed to Fermyon Cloud!"

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	cd cloudflare-worker-rust && cargo clean && rm -rf pkg pkg-sdk
	cd fermyon-spin-rust && cargo clean
	@echo "✅ Clean complete"

# Quick test of both local servers
test-all:
	@echo "Testing all local servers..."
	@echo "\n=== Cloudflare Worker ==="
	@$(MAKE) test-cloudflare || true
	@echo "\n=== Fermyon Spin ==="
	@$(MAKE) test-spin || true

# Full workflow
workflow-cloudflare: build-cloudflare
	@echo "Full Cloudflare workflow..."
	@echo "1. Starting dev server in background..."
	@cd cloudflare-worker-rust && wrangler dev > /dev/null 2>&1 & echo $$! > /tmp/wrangler.pid
	@sleep 5
	@echo "2. Running tests..."
	@$(MAKE) test-cloudflare
	@echo "3. Stopping dev server..."
	@kill `cat /tmp/wrangler.pid` 2>/dev/null || true
	@rm -f /tmp/wrangler.pid
	@echo "✅ Workflow complete"

workflow-spin: build-spin
	@echo "Full Spin workflow..."
	@echo "1. Starting Spin server in background..."
	@cd fermyon-spin-rust && spin up > /dev/null 2>&1 & echo $$! > /tmp/spin.pid
	@sleep 3
	@echo "2. Running tests..."
	@$(MAKE) test-spin
	@echo "3. Stopping Spin server..."
	@kill `cat /tmp/spin.pid` 2>/dev/null || true
	@rm -f /tmp/spin.pid
	@echo "✅ Workflow complete"