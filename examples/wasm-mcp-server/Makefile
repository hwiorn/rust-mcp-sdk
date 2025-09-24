# Makefile for WASM MCP Server
# Build once, deploy everywhere

.PHONY: help build clean deploy-cloudflare deploy-fermyon deploy-all test-all

help:
	@echo "╔══════════════════════════════════════════════════════════╗"
	@echo "║      WASM MCP Server - Write Once, Run Everywhere        ║"
	@echo "╚══════════════════════════════════════════════════════════╝"
	@echo ""
	@echo "Build Commands:"
	@echo "  make build              - Build WASM module"
	@echo "  make clean              - Clean build artifacts"
	@echo ""
	@echo "Deployment Commands:"
	@echo "  make deploy-cloudflare  - Deploy to Cloudflare Workers"
	@echo "  make deploy-fermyon     - Deploy to Fermyon Spin"
	@echo "  make deploy-all         - Deploy to all platforms"
	@echo ""
	@echo "Testing:"
	@echo "  make test-all           - Test all deployments"

# Build the core WASM module
build:
	@echo "Building WASM module..."
	cargo build --target wasm32-unknown-unknown --release
	@echo "✅ Core WASM module built"

# Clean all build artifacts
clean:
	cargo clean
	rm -rf deployments/cloudflare/pkg
	rm -rf deployments/cloudflare/build
	@echo "✅ Cleaned all artifacts"

# Deploy to Cloudflare Workers
deploy-cloudflare: build
	@echo "Deploying to Cloudflare Workers..."
	cd deployments/cloudflare && make deploy
	@echo "✅ Deployed to Cloudflare"

# Deploy to Fermyon Spin
deploy-fermyon: build
	@echo "Deploying to Fermyon Spin..."
	cd deployments/fermyon-spin && make deploy
	@echo "✅ Deployed to Fermyon"

# Deploy to all platforms
deploy-all: deploy-cloudflare deploy-fermyon
	@echo "✅ Deployed to all platforms!"

# Test all deployments
test-all:
	@echo "Testing Cloudflare deployment..."
	@curl -s -X POST https://mcp-sdk-worker.guy-ernest.workers.dev \
		-H "Content-Type: application/json" \
		-d '{"jsonrpc":"2.0","id":"1","method":"tools/list","params":{}}' \
		| jq -e '.result.tools' > /dev/null && echo "✅ Cloudflare: OK" || echo "❌ Cloudflare: FAILED"
	@echo ""
	@echo "Testing Fermyon deployment..."
	@curl -s -X POST https://mcp-fermyon-spin-3juc7zc4.fermyon.app/ \
		-H "Content-Type: application/json" \
		-d '{"jsonrpc":"2.0","id":"1","method":"tools/list","params":{}}' \
		| jq -e '.result.tools' > /dev/null && echo "✅ Fermyon: OK" || echo "❌ Fermyon: FAILED"