# WASM Target Documentation

## Overview

The pmcp SDK supports multiple WebAssembly compilation targets, each suited for different deployment environments.

## Compilation Targets

### 1. `wasm32-unknown-unknown` (Browser/Workers)

**Use Case**: Cloudflare Workers, Web browsers, serverless platforms without WASI support

**Features**:
- No system interface dependencies
- Suitable for sandboxed environments
- Uses `WasmServerCore` for MCP protocol handling
- Limited to pure computation (no file I/O, no system calls)

**Build Command**:
```bash
cargo build --target wasm32-unknown-unknown --no-default-features --features wasm
```

**Example**: `examples/cloudflare-worker-mcp/`

### 2. `wasm32-wasi` (WASI Preview 1)

**Use Case**: WASI-compliant runtimes, Wasmtime, Wasmer, Node.js with WASI support

**Features**:
- Access to WASI system interface
- File system access (sandboxed)
- Environment variables
- Basic networking (depending on runtime)

**Build Command**:
```bash
cargo build --target wasm32-wasi --no-default-features --features wasm
```

### 3. `wasm32-wasi-preview2` (Component Model)

**Use Case**: WASI HTTP/Preview2 environments, component-based architectures

**Features**:
- Component model support
- `wasi:http` interface for HTTP handling
- Advanced capabilities through world definitions
- Uses `wasi_adapter` with `feature = "wasi-http"`

**Build Command**:
```bash
cargo build --target wasm32-wasi --no-default-features --features "wasm wasi-http"
```

## Module Architecture

### WASM-Compatible Modules

When compiled for WASM targets, the SDK provides:

- **`server::wasm_core`**: Minimal MCP server implementation for pure WASM environments
- **`server::wasi_adapter`**: HTTP adapter for WASI environments
- **`server::ProtocolHandler`** (WASM version): Simplified trait without native dependencies

### Native-Only Modules

These modules are NOT available in WASM builds:

- **`server::core::ServerCore`**: Full server with auth, cancellation, subscriptions
- **`server::builder::ServerBuilder`**: Builder pattern with native dependencies
- **`server::adapters`**: Transport adapters requiring native I/O
- **Handler traits**: `ToolHandler`, `PromptHandler`, `ResourceHandler` (use simplified approach in WASM)

## Choosing the Right Target

| Environment | Target | Module to Use |
|------------|--------|--------------|
| Cloudflare Workers | `wasm32-unknown-unknown` | `WasmServerCore` |
| Vercel Edge Functions | `wasm32-unknown-unknown` | `WasmServerCore` |
| Fastly Compute@Edge | `wasm32-wasi` | `wasi_adapter` |
| Wasmtime/Wasmer CLI | `wasm32-wasi` | `wasi_adapter` |
| WASI HTTP Component | `wasm32-wasi` + `wasi-http` feature | `wasi_adapter::wasi_http_world` |

## Example: Cloudflare Worker with SDK

```rust
use pmcp::server::wasm_core::WasmServerCore;
use pmcp::server::wasi_adapter::WasiHttpAdapter;
use pmcp::server::ProtocolHandler;

#[worker::event(fetch)]
pub async fn main(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let mut server = WasmServerCore::new(
        "my-server".to_string(),
        "1.0.0".to_string(),
    );
    
    // Add tools
    server.add_tool("my-tool".to_string(), "Description".to_string(), |args| {
        // Tool implementation
        Ok(serde_json::json!({"result": "success"}))
    });
    
    // Process request
    let body = req.text().await?;
    let handler = Arc::new(server);
    let adapter = WasiHttpAdapter::new();
    let response = adapter.handle_request(handler, body).await?;
    
    Response::ok(response)
}
```

## Limitations in WASM

1. **No async trait methods with Send bounds** - Use `?Send` for WASM
2. **No tokio runtime** - Use browser/runtime-provided async
3. **No file system access** (in `wasm32-unknown-unknown`)
4. **No direct network access** - Use fetch API or runtime-provided networking
5. **Limited concurrency** - Single-threaded execution model

## Feature Flags

- `wasm`: Enable WASM-compatible modules
- `wasi-http`: Enable WASI HTTP world bindings (requires WASI target)
- `wasm-tokio`: Use tokio-compatible runtime for WASM (experimental)