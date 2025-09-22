# Cloudflare Worker MCP Example - SDK Version

This directory contains two implementations of an MCP server for Cloudflare Workers:

1. **Original** (`src/lib.rs`): Reimplements MCP types (demonstrates the architecture)
2. **SDK-based** (`src/lib_sdk.rs`): Uses the pmcp SDK directly (recommended)

## SDK-Based Implementation

The SDK-based version demonstrates using the pmcp SDK in Cloudflare Workers now that WASM compilation is supported.

### Setup

1. **Use the SDK Cargo configuration**:
```bash
cp Cargo_sdk.toml Cargo.toml
```

2. **Build the WASM module**:
```bash
wrangler build
```

Or manually:
```bash
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen target/wasm32-unknown-unknown/release/cloudflare_worker_mcp_sdk.wasm \
  --out-dir build \
  --target bundler
```

3. **Update wrangler.toml**:

For the Rust SDK version, change the main entry:
```toml
[build]
command = "cargo build --target wasm32-unknown-unknown --release"

[[build.upload]]
dir = "build"
format = "modules"

[build.upload.rules]
type = "ESModule"
globs = ["**/*.wasm"]

main = "build/cloudflare_worker_mcp_sdk.js"
```

### Routing

The SDK example serves MCP on `POST /mcp` endpoint. Update your client to use this endpoint:

```javascript
// Client configuration
const response = await fetch('https://your-worker.workers.dev/mcp', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify(mcpRequest)
});
```

### Testing Locally

```bash
# Start local development server
wrangler dev

# In another terminal, test with curl
curl -X POST http://localhost:8787/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "capabilities": {},
      "clientInfo": {
        "name": "test-client",
        "version": "1.0.0"
      }
    }
  }'
```

### Deployment

```bash
wrangler deploy
```

## Differences from Original Implementation

| Aspect | Original (`lib.rs`) | SDK-based (`lib_sdk.rs`) |
|--------|-------------------|------------------------|
| Types | Reimplemented | Uses pmcp types |
| Maintenance | Requires updates | Automatically updated with SDK |
| Features | Minimal | Full MCP protocol support |
| Size | Smaller | Slightly larger (includes SDK) |
| Routing | POST / | POST /mcp |

## Switching Between Implementations

To switch implementations:

1. **For original**: Use `Cargo.toml` and ensure `lib.rs` is the lib entry
2. **For SDK**: Use `Cargo_sdk.toml` and rename/link `lib_sdk.rs` as `lib.rs`

Or maintain both and use different Worker names:
```bash
# Deploy original
wrangler deploy --name mcp-worker-original

# Deploy SDK version  
wrangler deploy --name mcp-worker-sdk --config wrangler-sdk.toml
```

## Architecture Notes

The SDK version uses:
- `WasmServerCore`: WASM-compatible MCP server implementation
- `WasiHttpAdapter`: HTTP request/response adapter
- Simplified handler approach (no async traits with Send bounds)

This is the recommended approach for new MCP servers on Cloudflare Workers.