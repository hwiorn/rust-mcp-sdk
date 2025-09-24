# Makefile for Fermyon Spin Deployment

.PHONY: help build deploy test clean

help:
	@echo "Fermyon Spin MCP Server Deployment"
	@echo ""
	@echo "Commands:"
	@echo "  make build   - Build WASM module for Spin"
	@echo "  make deploy  - Deploy to Fermyon Cloud"
	@echo "  make test    - Test the deployment"
	@echo "  make clean   - Clean build artifacts"

build:
	@echo "Building for Fermyon Spin..."
	cd ../.. && cargo build --target wasm32-wasip1 --release
	@echo "✅ Built WASM module"

deploy: build
	@echo "Deploying to Fermyon Cloud..."
	spin deploy
	@echo "✅ Deployed successfully"

test:
	@echo "Testing Fermyon Spin deployment..."
	@curl -X POST https://mcp-fermyon-spin-3juc7zc4.fermyon.app/ \
		-H "Content-Type: application/json" \
		-d '{"jsonrpc":"2.0","id":"1","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}' \
		| jq '.result.serverInfo'

clean:
	cd ../.. && cargo clean
	@echo "✅ Cleaned build artifacts"