# MCP SDK Cloudflare Worker Example

This example demonstrates using the pmcp SDK in a Cloudflare Worker using the official Rust Workers support.

## Prerequisites

1. Install Rust and the WASM target:
```bash
rustup target add wasm32-unknown-unknown
```

2. Install worker-build:
```bash
cargo install worker-build
```

3. Install Wrangler:
```bash
npm install -g wrangler
```

## Local Development

```bash
# Build and run locally
wrangler dev

# Test the worker
curl -X POST http://localhost:8787 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "calculator",
      "arguments": {
        "operation": "add",
        "a": 25,
        "b": 17
      }
    }
  }'
```

## Deployment

```bash
# Login to Cloudflare (first time only)
wrangler login

# Deploy the worker
wrangler deploy

# Your worker will be available at:
# https://mcp-sdk-worker.<your-subdomain>.workers.dev
```

## Available Tools

### 1. Calculator
Performs arithmetic operations.

```json
{
  "name": "calculator",
  "arguments": {
    "operation": "add|subtract|multiply|divide",
    "a": <number>,
    "b": <number>
  }
}
```

### 2. Weather
Returns weather information (mock data).

```json
{
  "name": "weather",
  "arguments": {
    "location": "city name"
  }
}
```

### 3. System Info
Returns system information.

```json
{
  "name": "system_info",
  "arguments": {}
}
```

## MCP Protocol Examples

### List Available Tools
```bash
curl -X POST https://your-worker.workers.dev \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list",
    "params": {}
  }'
```

### Call a Tool
```bash
curl -X POST https://your-worker.workers.dev \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "calculator",
      "arguments": {
        "operation": "multiply",
        "a": 12,
        "b": 7
      }
    }
  }'
```

## Architecture

This worker uses:
- **pmcp SDK**: The Rust MCP SDK compiled to WASM
- **WasmServerCore**: WASM-compatible MCP server implementation
- **WasiHttpAdapter**: HTTP request/response adapter
- **workers-rs**: Cloudflare's official Rust Workers framework

## Key Features

- ✅ No initialization required (stateless operation)
- ✅ Full MCP protocol support through SDK
- ✅ CORS headers for browser access
- ✅ Proper error handling and logging
- ✅ Optimized for size with LTO and wasm-opt

## Troubleshooting

If you get build errors:
1. Make sure you have the wasm32-unknown-unknown target installed
2. Ensure worker-build is installed: `cargo install worker-build`
3. Clear the build directory: `rm -rf build/`

If deployment fails:
1. Make sure you're logged in: `wrangler login`
2. Check your account has Workers enabled
3. Try a different worker name in wrangler.toml