# Cloudflare Worker MCP Example - Current State

## Current Configuration

The example is now configured to use the **SDK-based implementation**:

- **Cargo.toml**: Set up for `cloudflare-worker-mcp-sdk` with pmcp dependency
- **src/lib.rs**: SDK-based implementation using `WasmServerCore`
- **src/lib_original.rs**: Original implementation (without SDK) preserved as backup

## Directory Structure

```
examples/cloudflare-worker-mcp/
├── Cargo.toml                 # SDK version (active)
├── Cargo_sdk.toml             # SDK version (backup - same as Cargo.toml)
├── src/
│   ├── lib.rs                 # SDK implementation (active)
│   ├── lib_original.rs        # Original implementation (backup)
│   └── types.rs               # Types for original implementation
├── src/bin/
│   └── test-client.rs         # Test client (native only)
└── worker-with-tools.js       # JavaScript worker (alternative)
```

## Building

From the **main SDK directory** (`/Users/guy/Development/mcp/sdk/rust-mcp-sdk/`):

```bash
# Build SDK for WASM first
cargo build --target wasm32-unknown-unknown --no-default-features --features wasm

# Build the Cloudflare Worker
make cloudflare-sdk-build

# Or manually:
cd examples/cloudflare-worker-mcp
cargo build --target wasm32-unknown-unknown --release --lib
```

## Key Points

1. **Why `--lib` flag?**: The test-client binary requires native dependencies (reqwest, tokio) that don't compile for WASM. Using `--lib` builds only the library.

2. **Endpoint**: The SDK version serves on `POST /mcp` (not `/` like the original)

3. **Two Implementations**: 
   - Original shows MCP protocol internals
   - SDK version is production-ready

## Testing Locally

```bash
# From main SDK directory
make cloudflare-sdk-dev

# In another terminal
curl -X POST http://localhost:8787/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'
```

## Switching Back to Original

If you need the original (non-SDK) version:

```bash
cd examples/cloudflare-worker-mcp
mv src/lib.rs src/lib_sdk_backup.rs
mv src/lib_original.rs src/lib.rs

# Also need original Cargo.toml (would need to be recreated)
```

## Why Keep Both?

- **Original**: Educational, minimal size, proves architecture
- **SDK**: Production use, maintained types, full protocol support