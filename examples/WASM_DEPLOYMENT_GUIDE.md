# WASM MCP Server Deployment Guide

This guide walks you through building, deploying, and testing the MCP SDK WASM examples.

## Prerequisites

### Common Requirements
```bash
# Install Rust and WASM targets
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
rustup target add wasm32-wasip1

# Install wasm-pack for building WASM modules
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

---

## Example 1: Cloudflare Workers

### Prerequisites
```bash
# Install Node.js (if not already installed)
# macOS: brew install node
# Linux: Use your package manager or nvm

# Install Cloudflare Wrangler CLI
npm install -g wrangler

# Login to Cloudflare (you'll need a free account)
wrangler login
```

### Build & Deploy

```bash
# Navigate to the Cloudflare example
cd examples/cloudflare-worker-rust

# Option 1: Using the Makefile (Recommended)
make build-sdk     # Build the SDK-backed version
make deploy        # Deploy to Cloudflare

# Option 2: Manual build
cargo build --target wasm32-unknown-unknown --release
wrangler deploy
```

### Local Testing

```bash
# Start local development server
wrangler dev

# In another terminal, test the MCP server
# Initialize
curl -X POST http://localhost:8787 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "1",
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "clientInfo": {
        "name": "test-client",
        "version": "1.0.0"
      }
    }
  }' | jq .

# List tools
curl -X POST http://localhost:8787 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "2",
    "method": "tools/list",
    "params": {}
  }' | jq .

# Call calculator tool
curl -X POST http://localhost:8787 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "3",
    "method": "tools/call",
    "params": {
      "name": "calculator",
      "arguments": {
        "operation": "add",
        "a": 10,
        "b": 20
      }
    }
  }' | jq .
```

### Production Deployment

```bash
# Deploy to Cloudflare Workers (requires account)
wrangler deploy

# Your worker will be available at:
# https://mcp-server-sdk.<your-subdomain>.workers.dev

# Test production endpoint
curl -X POST https://mcp-server-sdk.<your-subdomain>.workers.dev \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":"1","method":"initialize","params":{"protocolVersion":"2024-11-05","clientInfo":{"name":"test","version":"1.0.0"}}}' | jq .
```

### Troubleshooting Cloudflare

1. **Build errors**: Ensure you're using `wasm32-unknown-unknown` target
2. **Deployment errors**: Check `wrangler whoami` to verify login
3. **Runtime errors**: Use `wrangler tail` to see live logs

---

## Example 2: Fermyon Spin

### Prerequisites
```bash
# Install Fermyon Spin
curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash
sudo mv spin /usr/local/bin/

# Verify installation
spin --version
```

### Build & Deploy

```bash
# Navigate to the Spin example
cd examples/fermyon-spin-rust

# Build the WASM component
spin build
# OR manually:
cargo build --target wasm32-wasip1 --release

# Run locally
spin up
```

### Local Testing

```bash
# Spin starts on http://localhost:3000 by default

# Initialize
curl -X POST http://localhost:3000 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "1",
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "clientInfo": {
        "name": "test-client",
        "version": "1.0.0"
      }
    }
  }' | jq .

# List tools
curl -X POST http://localhost:3000 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "2",
    "method": "tools/list",
    "params": {}
  }' | jq .

# Call the add tool
curl -X POST http://localhost:3000 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "3",
    "method": "tools/call",
    "params": {
      "name": "add",
      "arguments": {
        "a": 15,
        "b": 25
      }
    }
  }' | jq .

# Call the reverse tool
curl -X POST http://localhost:3000 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "4",
    "method": "tools/call",
    "params": {
      "name": "reverse",
      "arguments": {
        "text": "Hello WASM!"
      }
    }
  }' | jq .

# Get environment info
curl -X POST http://localhost:3000 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "5",
    "method": "tools/call",
    "params": {
      "name": "environment",
      "arguments": {}
    }
  }' | jq .
```

### Deploy to Fermyon Cloud

```bash
# Login to Fermyon Cloud (free tier available)
spin cloud login

# Deploy the application
spin deploy

# Your app will be available at:
# https://<app-name>.fermyon.app

# Test production endpoint
curl -X POST https://<app-name>.fermyon.app \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":"1","method":"tools/list","params":{}}' | jq .
```

### Troubleshooting Spin

1. **Build errors**: Make sure you have `wasm32-wasip1` target installed
2. **Runtime errors**: Check `spin up --verbose` for detailed logs
3. **Port conflicts**: Use `spin up --listen 127.0.0.1:4000` for custom port

---

## Using the Makefile

A comprehensive Makefile is provided in the `examples/` directory:

```bash
cd examples/

# Install dependencies
make install-deps

# Build examples
make build-cloudflare    # Build Cloudflare example
make build-spin          # Build Spin example
make build-all           # Build both

# Run development servers
make dev-cloudflare      # Start Cloudflare dev server
make dev-spin            # Start Spin dev server

# Test examples (requires dev servers running)
make test-cloudflare     # Test Cloudflare
make test-spin           # Test Spin
make test-all            # Test both

# Deploy to production
make deploy-cloudflare   # Deploy to Cloudflare Workers
make deploy-spin         # Deploy to Fermyon Cloud

# Run full workflow (build, test, cleanup)
make workflow-cloudflare # Full Cloudflare workflow
make workflow-spin       # Full Spin workflow

# Clean build artifacts
make clean
```

---

## Comparison

| Feature | Cloudflare Workers | Fermyon Spin |
|---------|-------------------|--------------|
| **Target** | `wasm32-unknown-unknown` | `wasm32-wasip1` |
| **Local Dev** | `wrangler dev` | `spin up` |
| **Default Port** | 8787 | 3000 |
| **Deploy Command** | `wrangler deploy` | `spin deploy` |
| **Free Tier** | Yes (100k req/day) | Yes |
| **Global Edge** | Yes | Yes (via Fermyon Cloud) |
| **WASI Standard** | No (custom) | Yes (WASI P1) |

## Common Issues & Solutions

### Issue: "error: target not found"
**Solution**: Install the required target
```bash
rustup target add wasm32-unknown-unknown  # For Cloudflare
rustup target add wasm32-wasip1           # For Spin
```

### Issue: "wasm-pack not found"
**Solution**: Install wasm-pack
```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

### Issue: CORS errors in browser
**Solution**: Both examples include CORS headers. For OPTIONS preflight:
- Cloudflare: Handled in `src/lib.rs`
- Spin: Handled in `src/lib.rs`

### Issue: "method not found"
**Solution**: Ensure you're using the correct MCP protocol format:
- Method names like `tools/list`, not `listTools`
- Include `jsonrpc: "2.0"` in all requests

## Next Steps

After successful deployment:

1. **Integration**: Connect your MCP client to the deployed endpoint
2. **Customization**: Add your own tools, resources, and prompts
3. **Monitoring**: Use platform tools (Cloudflare Analytics, Spin logs)
4. **Scaling**: Both platforms auto-scale based on demand

## Summary

Both examples demonstrate the same MCP server logic running on different WASI platforms:
- **Cloudflare Workers**: Best for edge deployment with global distribution
- **Fermyon Spin**: Best for standard WASI compliance and portability

The key achievement is that the MCP logic (`WasmMcpServer`) remains identical across both platforms, with only thin platform-specific wrappers.