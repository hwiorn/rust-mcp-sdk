# Makefile for MCP Server Cloudflare Worker Deployment

.PHONY: help install check dev build deploy test clean tail logs production

# Default target
help:
	@echo "MCP Server Cloudflare Worker - Available Targets"
	@echo "================================================"
	@echo "  make install      - Install all prerequisites"
	@echo "  make check        - Check prerequisites are installed"
	@echo "  make build        - Build the WASM binary"
	@echo "  make dev          - Run development server locally"
	@echo "  make test         - Run tests"
	@echo "  make test-client  - Run test client against local server"
	@echo "  make deploy       - Deploy to Cloudflare (staging)"
	@echo "  make production   - Deploy to production"
	@echo "  make tail         - Stream live logs from worker"
	@echo "  make logs         - View recent logs"
	@echo "  make clean        - Clean build artifacts"

# Install prerequisites
install:
	@echo "📦 Installing prerequisites..."
	@rustup target add wasm32-unknown-unknown
	@cargo install wasm-pack || true
	@npm install -g wrangler
	@echo "✅ Prerequisites installed"

# Check prerequisites
check:
	@echo "🔍 Checking prerequisites..."
	@command -v cargo >/dev/null 2>&1 || { echo "❌ cargo not found"; exit 1; }
	@command -v rustc >/dev/null 2>&1 || { echo "❌ rustc not found"; exit 1; }
	@command -v wrangler >/dev/null 2>&1 || { echo "❌ wrangler not found"; exit 1; }
	@command -v wasm-pack >/dev/null 2>&1 || { echo "❌ wasm-pack not found"; exit 1; }
	@rustup target list | grep -q "wasm32-unknown-unknown (installed)" || { echo "❌ wasm32-unknown-unknown target not installed"; exit 1; }
	@echo "✅ All prerequisites installed"
	@wrangler whoami || { echo "⚠️  Not logged in to Cloudflare. Run: wrangler login"; exit 1; }

# Build the worker
build: check
	@echo "🔨 Building Worker..."
	@wasm-pack build --target web --out-dir build
	@echo "✅ Build complete"

# Run development server
dev: build
	@echo "🚀 Starting development server..."
	@wrangler dev --local

# Run development server (remote)
dev-remote: build
	@echo "🚀 Starting development server (remote)..."
	@wrangler dev --local=false

# Run tests
test:
	@echo "🧪 Running tests..."
	@cargo test
	@echo "✅ Tests passed"

# Run test client against local server
test-client:
	@echo "🧪 Running test client..."
	@cargo build --bin test-client
	@cargo run --bin test-client http://localhost:8787

# Run test client against deployed server
test-deployed:
	@echo "🧪 Running test client against deployed server..."
	@cargo build --bin test-client
	@cargo run --bin test-client

# Deploy to staging
deploy: build
	@echo "🚀 Deploying to Cloudflare Workers (staging)..."
	@wrangler deploy
	@echo "✅ Deployed to staging"
	@echo "📍 URL: https://mcp-server.$$(wrangler whoami | grep -oE '[a-z0-9-]+\.workers\.dev' | head -1)"

# Deploy to production
production: build
	@echo "🚀 Deploying to Cloudflare Workers (production)..."
	@wrangler deploy --env production
	@echo "✅ Deployed to production"

# Stream logs
tail:
	@echo "📊 Streaming logs..."
	@wrangler tail

# View recent logs
logs:
	@echo "📊 Fetching recent logs..."
	@wrangler tail --once

# Clean build artifacts
clean:
	@echo "🧹 Cleaning build artifacts..."
	@rm -rf build/
	@rm -rf target/wasm32-unknown-unknown/
	@cargo clean
	@echo "✅ Clean complete"

# Quality gates (following project standards)
quality-gate: format-check lint-check

format-check:
	@echo "🎨 Checking formatting..."
	@cargo fmt --check

lint-check:
	@echo "🔍 Running clippy..."
	@cargo clippy -- -D warnings

# Full CI pipeline
ci: check quality-gate test build
	@echo "✅ CI pipeline complete"

# Development workflow with quality checks
dev-quality: quality-gate dev

# Integration test (build, deploy to dev, test, cleanup)
integration-test: build
	@echo "🧪 Running integration test..."
	@wrangler dev --local=false & echo $$! > .dev.pid
	@sleep 5
	@cargo run --bin test-client http://localhost:8787 || (kill $$(cat .dev.pid); rm .dev.pid; exit 1)
	@kill $$(cat .dev.pid)
	@rm .dev.pid
	@echo "✅ Integration test complete"

# KV namespace setup (optional)
setup-kv:
	@echo "🗄️ Setting up KV namespace..."
	@wrangler kv:namespace create "MCP_STORE"
	@echo "📝 Add the namespace ID to wrangler.toml"

# List all workers
list-workers:
	@echo "📋 Listing all workers..."
	@wrangler deployments list

# Get worker metrics
metrics:
	@echo "📊 Fetching worker metrics..."
	@wrangler tail --format json --once | jq '.[] | {timestamp, outcome, duration}'

# Rollback to previous version
rollback:
	@echo "⏮️ Rolling back to previous version..."
	@wrangler rollback
	@echo "✅ Rollback complete"