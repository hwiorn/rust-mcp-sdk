# Cloudflare Workers Deployment

This folder contains the deployment configuration for running the WASM MCP server on Cloudflare Workers.

## Files

- **wrangler.toml** - Cloudflare Workers configuration
- **worker-rust.js** - JavaScript wrapper that initializes WASM
- **worker.js** - JavaScript fallback implementation
- **Makefile** - Build and deployment commands
- **DEPLOYMENT.md** - Detailed deployment guide
- **ARCHITECTURE.md** - Technical architecture documentation

## Quick Deploy

```bash
# Build WASM module
make build

# Deploy to Cloudflare
make deploy

# Test deployment
make test-prod
```

## Configuration

The `wrangler.toml` file specifies:
- Worker name: `mcp-sdk-worker`
- Entry point: `worker-rust.js` (WASM wrapper)
- Compatibility date: `2024-11-05`

## Build Process

1. **wasm-pack** builds the parent Rust code to WASM
2. **worker-rust.js** wraps and initializes the WASM module
3. **wrangler** deploys to Cloudflare's edge network

## Key Differences from Fermyon Spin

- Uses `wasm32-unknown-unknown` target (not WASI)
- Requires JavaScript wrapper for initialization
- Global edge deployment (200+ locations)
- KV and Durable Objects for state

## Troubleshooting

- If build fails: Use `--no-opt` flag with wasm-pack
- If runtime fails: Check the JavaScript wrapper initialization
- For CORS issues: Headers are set in the Rust code

## Live Deployment

🌐 https://mcp-sdk-worker.guy-ernest.workers.dev