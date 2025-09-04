.PHONY: build run-stdio run-http test-client test-stdio test-http inspector clean help

# Default target
help:
	@echo "OAuth Basic Example - Available Commands"
	@echo "========================================"
	@echo ""
	@echo "Building:"
	@echo "  make build          - Build the OAuth server"
	@echo "  make build-release  - Build optimized release version"
	@echo ""
	@echo "Running:"
	@echo "  make run-stdio      - Run server in STDIO mode"
	@echo "  make run-http       - Run server in HTTP mode (port 8080)"
	@echo "  make run-http-port PORT=3000 - Run on custom port"
	@echo ""
	@echo "Testing:"
	@echo "  make test-client    - Run the HTTP client test"
	@echo "  make test-stdio     - Test STDIO protocol directly"
	@echo "  make test-http      - Test HTTP with curl"
	@echo "  make test-all       - Run all tests"
	@echo ""
	@echo "MCP Inspector:"
	@echo "  make inspector      - Launch with MCP Inspector (STDIO)"
	@echo "  make inspector-http - Launch with MCP Inspector (HTTP)"
	@echo ""
	@echo "Utilities:"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make watch          - Watch and rebuild on changes"

# Build targets
build:
	@echo "Building OAuth server..."
	@cargo build --package oauth-basic

build-release:
	@echo "Building OAuth server (release)..."
	@cargo build --package oauth-basic --release

# Run targets
run-stdio: build
	@echo "Starting OAuth server in STDIO mode..."
	@./../../target/debug/oauth-basic stdio

run-http: build
	@echo "Starting OAuth server in HTTP mode on port 8080..."
	@./../../target/debug/oauth-basic http 8080

run-http-port: build
	@echo "Starting OAuth server in HTTP mode on port $(PORT)..."
	@./../../target/debug/oauth-basic http $(PORT)

# Test targets
test-client: build
	@echo "Running HTTP client test..."
	@cargo run --bin test-client

test-stdio: build
	@echo "Testing STDIO protocol..."
	@echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}' | \
		./../../target/debug/oauth-basic stdio 2>/dev/null | \
		head -1 | jq . || echo "Error: No response or invalid JSON"

test-http: build
	@echo "Testing HTTP server..."
	@# Start server in background
	@./../../target/debug/oauth-basic http 8080 > /tmp/oauth-test.log 2>&1 & \
		SERVER_PID=$$!; \
		sleep 2; \
		echo "Testing server on PID $$SERVER_PID..."; \
		curl -s -X POST http://localhost:8080/ \
			-H "Content-Type: application/json" \
			-H "Accept: text/event-stream" \
			-d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"curl","version":"1.0.0"}}}' | \
			head -5; \
		kill $$SERVER_PID 2>/dev/null; \
		echo "Test complete"

test-all: test-stdio test-client
	@echo "All tests completed"

# MCP Inspector targets
inspector: build
	@echo "Launching MCP Inspector with STDIO transport..."
	@echo "Server binary: $(shell pwd)/../../target/debug/oauth-basic"
	@npx @modelcontextprotocol/inspector $(shell pwd)/../../target/debug/oauth-basic stdio

inspector-http: build
	@echo "Starting HTTP server for MCP Inspector..."
	@./../../target/debug/oauth-basic http 8080 > /tmp/oauth-inspector.log 2>&1 & \
		echo "Server started on PID $$!"; \
		echo "Connect MCP Inspector to: http://localhost:8080"; \
		echo "Press Ctrl+C to stop the server"; \
		wait

# Utility targets
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean --package oauth-basic
	@rm -f /tmp/oauth-*.log

watch:
	@echo "Watching for changes..."
	@cargo watch -x "build --package oauth-basic"

# Kill any running OAuth servers
kill-servers:
	@echo "Stopping any running OAuth servers..."
	@pkill -f oauth-basic || true