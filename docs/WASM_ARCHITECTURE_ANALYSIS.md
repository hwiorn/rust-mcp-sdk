# WASM/WASI Architecture Analysis for MCP SDK

## Goal
Enable developers to build MCP servers with the Rust SDK that:
1. Focus on MCP tools, resources, and prompts logic
2. Are environment-agnostic 
3. Can be wrapped for any WASI environment (Cloudflare, Spin, Wasmtime, etc.)

## Current State Assessment

### ✅ Well Implemented

#### 1. **Basic WASM Compilation**
- SDK compiles successfully for `wasm32-unknown-unknown`
- Dependencies properly gated (reqwest native-only)
- Conditional compilation structure in place

#### 2. **Protocol Handler Abstraction**
```rust
pub trait ProtocolHandler {
    async fn handle_request(&self, id: RequestId, request: Request) -> JSONRPCResponse;
    async fn handle_notification(&self, notification: Notification) -> Result<()>;
}
```
- Clean interface between transport and protocol
- Stateless design suitable for serverless

#### 3. **WasiHttpAdapter**
- Converts HTTP requests to MCP messages
- Handles request/response transformation
- Session support (optional)

#### 4. **Tool Registration in WasmServerCore**
```rust
pub fn add_tool<F>(&mut self, name: String, description: String, handler: F)
where F: Fn(Value) -> Result<Value> + 'static
```
- Simple closure-based tool definition
- No async complexity in tool handlers

### ❌ Missing or Inadequate

#### 1. **Type Safety Lost in WASM**
**Current Issue**: WasmServerCore uses JSON values instead of typed MCP structures
```rust
// Current approach - loses type safety
match serde_json::to_value(&*client_request) {
    Ok(req_value) => {
        if let Some(method) = req_value.get("method").and_then(|m| m.as_str()) {
            match method {
                "initialize" => { /* manual JSON construction */ }
```

**Should Be**: Proper typed request handling
```rust
match client_request {
    ClientRequest::Initialize(params) => {
        // Typed handling
    }
    ClientRequest::ListTools(params) => {
        // Typed handling
    }
}
```

#### 2. **Tool Result Format**
**Current Issue**: Returns stringified JSON in text content
```rust
Ok(json!({
    "content": [{
        "type": "text",
        "text": serde_json::to_string(&result)?  // Stringified!
    }],
    "isError": false
}))
```

**Should Be**: Structured content types
```rust
CallToolResult {
    content: vec![Content::ToolResult(result)],
    is_error: false,
}
```

#### 3. **No Resource or Prompt Support in WASM**
- WasmServerCore only supports tools
- No resource handlers
- No prompt handlers
- No sampling support

#### 4. **Environment Coupling**
**Current Issue**: Direct dependency on specific frameworks
```rust
use worker::*;  // Cloudflare-specific

#[event(fetch)]  // Worker-specific macro
async fn main(req: Request, env: Env, ctx: Context) -> Result<Response>
```

**Should Be**: Environment-agnostic core with adapters

#### 5. **No Streaming Support**
- All responses are buffered
- No support for streaming tools or resources
- Important for large responses

## Proposed Architecture

### Layer 1: Core MCP Logic (Environment-Agnostic)

```rust
// Pure MCP server implementation
pub struct McpServer {
    tools: HashMap<String, Box<dyn Tool>>,
    resources: HashMap<String, Box<dyn Resource>>,
    prompts: HashMap<String, Box<dyn Prompt>>,
}

// Tool trait that's WASM-friendly
pub trait Tool: Send + Sync {
    fn execute(&self, args: Value) -> Result<ToolResult>;
    fn schema(&self) -> ToolSchema;
}

// Similar for resources and prompts
pub trait Resource: Send + Sync {
    fn read(&self, uri: &str) -> Result<ResourceContent>;
    fn list(&self) -> Result<Vec<ResourceInfo>>;
}
```

### Layer 2: WASI Adapter (Handles Protocol)

```rust
// Generic WASI adapter
pub struct WasiMcpAdapter<S: McpServer> {
    server: S,
}

impl<S> WasiMcpAdapter<S> {
    // Convert HTTP/stdin/socket to MCP
    pub fn handle_request(&self, input: &[u8]) -> Vec<u8> {
        // Protocol handling
    }
}
```

### Layer 3: Environment Bindings (Platform-Specific)

```rust
// Cloudflare wrapper
#[cfg(feature = "cloudflare")]
mod cloudflare {
    use worker::*;
    
    #[event(fetch)]
    async fn main(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
        let adapter = create_adapter();
        let body = req.bytes().await?;
        let response = adapter.handle_request(&body);
        Response::ok(response)
    }
}

// Spin wrapper  
#[cfg(feature = "spin")]
mod spin {
    use spin_sdk::http::{Request, Response};
    
    #[spin_sdk::http_component]
    fn handle(req: Request) -> Response {
        let adapter = create_adapter();
        let response = adapter.handle_request(req.body());
        Response::builder().body(response).build()
    }
}

// Wasmtime/WASI CLI wrapper
#[cfg(feature = "wasi-cli")]
mod wasi_cli {
    use std::io::{stdin, stdout, Read, Write};
    
    fn main() {
        let adapter = create_adapter();
        let mut input = Vec::new();
        stdin().read_to_end(&mut input).unwrap();
        let response = adapter.handle_request(&input);
        stdout().write_all(&response).unwrap();
    }
}
```

## Environment Comparison

### Cloudflare Workers
**Pros:**
- Massive scale, global edge network
- Good Rust support via workers-rs
- Free tier available

**Cons:**
- Complex build tooling (worker-build issues)
- Proprietary platform
- Limited WASI support (wasm32-unknown-unknown)

### Fermyon Spin
**Pros:**
- Purpose-built for WASM serverless
- Excellent Rust support
- Standard WASI components
- Simple deployment

**Cons:**
- Smaller ecosystem
- Less mature platform

### Fastly Compute@Edge
**Pros:**
- Good Rust support
- Edge deployment
- WASI-based

**Cons:**
- No free tier
- Proprietary platform

### Wasmtime/Wasmer (CLI)
**Pros:**
- Standard WASI implementation
- Local development friendly
- Open source

**Cons:**
- Not a deployment platform
- No built-in HTTP handling

## Recommendations

### 1. **Refactor WasmServerCore**
Create a properly typed MCP server that:
- Maintains type safety
- Supports all MCP features (tools, resources, prompts)
- Is truly environment-agnostic

### 2. **Create Environment Adapters**
Build thin wrappers for each platform:
- `pmcp-cloudflare` - Cloudflare Workers adapter
- `pmcp-spin` - Fermyon Spin adapter  
- `pmcp-wasi` - Generic WASI adapter

### 3. **Use Fermyon Spin for Examples**
Consider Spin for the primary WASM example because:
- Clean separation of concerns
- Standard WASI components
- Simple deployment model
- Better represents the "WASI standard" approach

### 4. **Provide Migration Path**
```rust
// Developer writes environment-agnostic code
struct MyTool;
impl Tool for MyTool {
    fn execute(&self, args: Value) -> Result<ToolResult> {
        // Business logic only
    }
}

// Choose deployment target with feature flag
#[cfg(feature = "deploy-cloudflare")]
use pmcp_cloudflare::deploy;

#[cfg(feature = "deploy-spin")]  
use pmcp_spin::deploy;

deploy![MyTool, MyResource, MyPrompt];  // Macro generates platform code
```

## Next Steps

1. **Design new McpServer trait** that's truly environment-agnostic
2. **Implement typed request handling** in WASM
3. **Create Spin example** as reference implementation
4. **Build adapter library** for each platform
5. **Document patterns** for MCP server developers

## Conclusion

The current SDK has good bones (protocol/transport separation, WASM compilation) but needs:
- Better type safety in WASM
- True environment independence  
- Support for all MCP features
- Clear abstraction layers

The focus should be on making MCP server development simple and portable, with deployment being a thin wrapper choice.