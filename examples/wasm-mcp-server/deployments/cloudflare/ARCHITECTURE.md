# Architecture Overview: MCP Server on Cloudflare Workers

## System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   MCP Client (Claude, etc.)              │
└─────────────────────────────┬───────────────────────────┘
                              │ HTTPS
                              ▼
┌─────────────────────────────────────────────────────────┐
│              Cloudflare Workers Runtime                  │
│  ┌─────────────────────────────────────────────────┐    │
│  │            worker-rust.js (Entry Point)         │    │
│  │  - Imports WASM module                          │    │
│  │  - Initializes once (memoized)                  │    │
│  │  - Routes requests to WASM                      │    │
│  └──────────────────┬──────────────────────────────┘    │
│                     │                                    │
│  ┌──────────────────▼──────────────────────────────┐    │
│  │         mcp_cloudflare_worker_bg.wasm           │    │
│  │                                                  │    │
│  │  ┌────────────────────────────────────────┐    │    │
│  │  │    Rust MCP Implementation (lib.rs)     │    │    │
│  │  │  - #[event(fetch)] handler              │    │    │
│  │  │  - WasmMcpServer (environment-agnostic) │    │    │
│  │  │  - JSON-RPC request parsing             │    │    │
│  │  │  - Tool implementations                 │    │    │
│  │  └────────────────────────────────────────┘    │    │
│  └──────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────┘
```

## Request Flow

1. **HTTP Request** → Cloudflare Edge
2. **worker-rust.js** → Receives fetch event
3. **WASM Init** → Ensures module is initialized (once)
4. **wasmFetch()** → Delegates to Rust handler
5. **Rust Handler** → Processes MCP protocol
6. **Response** → JSON-RPC response back to client

## Key Components

### 1. JavaScript Wrapper (worker-rust.js)

```javascript
import init, { fetch as wasmFetch } from './pkg/mcp_cloudflare_worker.js';
import wasmModule from './pkg/mcp_cloudflare_worker_bg.wasm';

// Memoized initialization
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

**Purpose:**
- Handle WASM module initialization
- Bridge between Workers runtime and Rust
- Error handling and recovery

### 2. Rust MCP Server (src/lib.rs)

```rust
#[event(fetch)]
async fn main(req: Request, env: Env, ctx: Context) -> Result<Response> {
    // 1. Parse HTTP request
    // 2. Route based on method (GET for info, POST for MCP)
    // 3. Create WasmMcpServer instance
    // 4. Process MCP request
    // 5. Return JSON-RPC response
}
```

**Key Features:**
- Stateless operation (new server per request)
- CORS support for browser clients
- Compatibility fixes (e.g., optional capabilities)

### 3. WasmMcpServer (SDK Component)

```rust
WasmMcpServer::builder()
    .name("cloudflare-mcp-worker")
    .version("1.0.0")
    .capabilities(...)
    .tool("calculator", SimpleTool::new(...))
    .build()
```

**Benefits:**
- Environment-agnostic (same code for Spin, Workers, etc.)
- Type-safe tool definitions
- Built-in protocol compliance

## Build Pipeline

```
src/lib.rs
    │
    ├─[cargo build]→ target/wasm32-unknown-unknown/
    │
    ├─[wasm-pack]→ pkg/
    │               ├── mcp_cloudflare_worker_bg.wasm
    │               ├── mcp_cloudflare_worker.js
    │               └── mcp_cloudflare_worker.d.ts
    │
    └─[wrangler]→ Cloudflare Workers
```

## Why This Architecture?

### 1. **wasm-pack vs worker-build**

We use `wasm-pack` because:
- ✅ Reliable WASM initialization
- ✅ Predictable JavaScript bindings
- ✅ Better debugging capabilities
- ✅ Works with `--no-opt` flag

We avoid `worker-build` because:
- ❌ Runtime error: `(void 0) is not a function`
- ❌ Minified output hard to debug
- ❌ Shim expectations don't match wasm-bindgen

### 2. **Explicit Initialization**

The JavaScript wrapper pattern ensures:
- WASM module initialized once
- Proper error handling
- Clear separation of concerns

### 3. **Stateless Design**

Each request creates a new server instance:
- No shared state issues
- Scales horizontally
- Matches Workers' execution model

## Protocol Implementation

### MCP Methods Supported

1. **initialize** - Protocol handshake
   ```json
   {"method": "initialize", "params": {"protocolVersion": "2024-11-05"}}
   ```

2. **tools/list** - List available tools
   ```json
   {"method": "tools/list", "params": {}}
   ```

3. **tools/call** - Execute a tool
   ```json
   {"method": "tools/call", "params": {"name": "calculator", "arguments": {}}}
   ```

### Request Parsing Flow

```rust
// 1. Extract JSON-RPC components
let method = request_value.get("method");
let params = request_value.get("params");

// 2. Route based on method
match method {
    "initialize" => {
        // Add missing capabilities if needed
        ClientRequest::Initialize(params)
    },
    "tools/list" => ClientRequest::ListTools(...),
    "tools/call" => ClientRequest::CallTool(...),
}

// 3. Process with WasmMcpServer
server.handle_request(id, request).await
```

## Performance Characteristics

### Bundle Size
- WASM: ~566KB
- JavaScript: ~32KB
- Total: ~600KB

### Latency
- Cold start: 50-200ms
- Warm request: 10-30ms
- Tool execution: 5-15ms

### Optimization Techniques

1. **Rust Compilation**
   ```toml
   [profile.release]
   lto = true           # Link-time optimization
   strip = true         # Remove debug symbols
   codegen-units = 1    # Single unit for better optimization
   opt-level = "z"      # Size optimization
   ```

2. **WASM Build**
   ```bash
   wasm-pack build --no-opt  # Skip wasm-opt to avoid issues
   ```

3. **Caching**
   - WASM module cached after first init
   - Cloudflare caches at edge

## Security Model

### Input Validation
```rust
// Parse and validate JSON-RPC
let request: ClientRequest = serde_json::from_value(params)
    .map_err(|e| Response::error("Invalid params", 400))?;
```

### CORS Headers
```rust
headers.set("Access-Control-Allow-Origin", "*")?;
headers.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS")?;
```

### Error Handling
```rust
// Graceful error responses
match result {
    Ok(response) => Response::ok(json!(response)),
    Err(e) => Response::error(&e.to_string(), 500)
}
```

## Deployment Model

### Multi-Region
Cloudflare automatically deploys to all edge locations

### Zero Downtime
```bash
wrangler deploy  # Atomic deployment
wrangler rollback # Quick rollback if needed
```

### Monitoring
```bash
wrangler tail  # Real-time logs
```

## Compatibility Notes

### MCP Protocol Version
- Supports: `2024-11-05`
- Handles missing `capabilities` field
- Compatible with mcp-tester (with workarounds)

### Browser Clients
Full CORS support enables browser-based MCP clients

### Other Platforms
Same `WasmMcpServer` runs on:
- Fermyon Spin (WASI)
- Deno Deploy
- Node.js
- Browser

## Future Enhancements

### Planned Features
1. Persistent state with KV storage
2. WebSocket support for streaming
3. Authentication middleware
4. Rate limiting
5. Metrics and observability

### Architecture Evolution
```
Current: Stateless per-request
Future:  Durable Objects for stateful sessions
```

## Comparison with Alternatives

### vs JavaScript Implementation
| Aspect | Rust/WASM | JavaScript |
|--------|-----------|------------|
| Performance | ⚡ Faster | 🐢 Slower |
| Type Safety | ✅ Compile-time | ⚠️ Runtime |
| Bundle Size | 📦 Larger (600KB) | 📄 Smaller (50KB) |
| Debugging | 🔧 Complex | 🎯 Simple |
| Code Reuse | ♻️ Cross-platform | 🔒 Workers-only |

### vs Fermyon Spin
| Aspect | Cloudflare Workers | Fermyon Spin |
|--------|-------------------|--------------|
| Deployment | Global edge | Single region |
| Cold Start | 50-200ms | 100-300ms |
| State | KV/Durable Objects | Built-in SQLite |
| Ecosystem | Web-focused | WASI-focused |

## Conclusion

This architecture provides:
- ✅ Production-ready MCP server
- ✅ Type-safe Rust implementation
- ✅ "Write once, run everywhere" capability
- ✅ Excellent performance at edge
- ✅ Full protocol compliance

The combination of Rust's safety, WASM's portability, and Cloudflare's global network creates a robust, scalable MCP server implementation.