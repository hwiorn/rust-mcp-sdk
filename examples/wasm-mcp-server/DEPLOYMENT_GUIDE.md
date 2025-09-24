# WASM MCP Server - Complete Deployment Guide

This guide shows how to deploy the WASM MCP server to different platforms.

## 📁 File Locations

### Core Implementation (Shared by All Platforms)
- `src/lib.rs` - The MCP server implementation in Rust
- `Cargo.toml` - Rust dependencies and build configuration

### Cloudflare Workers Files
Location: `deployments/cloudflare/`

| File | Purpose |
|------|---------|
| **wrangler.toml** | Cloudflare configuration (worker name, entry point) |
| **worker-rust.js** | JavaScript wrapper that initializes the WASM module |
| **worker.js** | JavaScript fallback implementation (for testing) |
| **Makefile** | Build and deployment commands |
| **DEPLOYMENT.md** | Detailed Cloudflare deployment instructions |
| **ARCHITECTURE.md** | Technical architecture documentation |
| **README.md** | Quick reference for Cloudflare deployment |

### Fermyon Spin Files
Location: `deployments/fermyon-spin/`

| File | Purpose |
|------|---------|
| **spin.toml** | Spin application configuration |
| **Makefile** | Build and deployment commands |
| **README.md** | Fermyon Spin deployment instructions |

## 🚀 Cloudflare Workers Deployment

### Prerequisites
```bash
# Install wrangler
npm install -g wrangler

# Install wasm-pack
cargo install wasm-pack

# Login to Cloudflare
wrangler login
```

### The JavaScript Wrapper (`worker-rust.js`)
This is the critical file that bridges WASM and Cloudflare Workers:

```javascript
import init, { fetch as wasmFetch } from './pkg/mcp_cloudflare_worker.js';
import wasmModule from './pkg/mcp_cloudflare_worker_bg.wasm';

// Initialize WASM once
let wasmInitialized = false;
async function ensureWasmInit() {
  if (!wasmInitialized) {
    await init(wasmModule);
    wasmInitialized = true;
  }
}

export default {
  async fetch(request, env, ctx) {
    await ensureWasmInit();
    return await wasmFetch(request, env, ctx);
  }
};
```

### Build and Deploy
```bash
cd deployments/cloudflare

# Build WASM with wasm-pack
wasm-pack build --target web --out-dir pkg --no-opt ../..

# Deploy to Cloudflare
wrangler deploy

# Or use the Makefile
make deploy
```

### Configuration (`wrangler.toml`)
```toml
name = "mcp-sdk-worker"
main = "worker-rust.js"  # Points to the JavaScript wrapper
compatibility_date = "2024-11-05"

[dev]
port = 8787
```

## 🌀 Fermyon Spin Deployment

### Prerequisites
```bash
# Install Spin
curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash

# Add WASI target
rustup target add wasm32-wasip1
```

### Configuration (`spin.toml`)
```toml
[application]
name = "mcp-fermyon-spin"
version = "0.1.0"

[[trigger.http]]
route = "/..."

[component.mcp-fermyon-spin]
source = "target/wasm32-wasip1/release/mcp_fermyon_spin.wasm"
```

### Build and Deploy
```bash
cd deployments/fermyon-spin

# Build for WASI
cargo build --target wasm32-wasip1 --release --manifest-path ../../Cargo.toml

# Deploy to Fermyon Cloud
spin deploy

# Or use the Makefile
make deploy
```

## 🔧 Platform Differences

### Build Targets
- **Cloudflare**: `wasm32-unknown-unknown` (pure WASM)
- **Fermyon Spin**: `wasm32-wasip1` (WASI)

### Entry Points
- **Cloudflare**: Uses `#[event(fetch)]` macro from `worker` crate
- **Fermyon Spin**: Uses `#[http_component]` macro from `spin-sdk`

### Initialization
- **Cloudflare**: Requires JavaScript wrapper for WASM initialization
- **Fermyon Spin**: Direct WASM execution, no wrapper needed

### State Management
- **Cloudflare**: KV storage, Durable Objects
- **Fermyon Spin**: Built-in SQLite, key-value stores

## 📝 Quick Commands

From the root `wasm-mcp-server/` directory:

```bash
# Build for all platforms
make build

# Deploy to Cloudflare
make deploy-cloudflare

# Deploy to Fermyon
make deploy-fermyon

# Deploy everywhere
make deploy-all

# Test all deployments
make test-all
```

## 🧪 Testing Deployments

### Test Cloudflare
```bash
curl -X POST https://mcp-sdk-worker.guy-ernest.workers.dev \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":"1","method":"tools/list","params":{}}'
```

### Test Fermyon
```bash
curl -X POST https://mcp-fermyon-spin-3juc7zc4.fermyon.app/ \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":"1","method":"tools/list","params":{}}'
```

## 📚 Additional Documentation

- **Cloudflare Details**: See `deployments/cloudflare/DEPLOYMENT.md`
- **Architecture Overview**: See `deployments/cloudflare/ARCHITECTURE.md`
- **Fermyon Details**: See `deployments/fermyon-spin/README.md`

## ❓ Common Issues

### Cloudflare: "void 0 is not a function"
- **Cause**: Using `worker-build` instead of `wasm-pack`
- **Solution**: Use the provided JavaScript wrapper with `wasm-pack`

### Cloudflare: Bulk memory operations error
- **Cause**: wasm-opt validation issues
- **Solution**: Build with `--no-opt` flag

### Fermyon: Component not found
- **Cause**: Wrong component name in spin.toml
- **Solution**: Ensure component name matches Cargo package name

## 🎯 Key Takeaway

The same `src/lib.rs` compiles and runs on both platforms. The only differences are:
1. Build target (wasm32-unknown-unknown vs wasm32-wasip1)
2. Platform adapter (thin wrapper for request/response)
3. Deployment configuration (wrangler.toml vs spin.toml)

This demonstrates true "write once, run everywhere" with WebAssembly!