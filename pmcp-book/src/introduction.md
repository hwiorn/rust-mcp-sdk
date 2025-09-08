# Introduction

The Model Context Protocol (MCP) is revolutionizing how AI applications interact with external systems, tools, and data sources. PMCP brings this power to the Rust ecosystem with uncompromising quality and performance.

## What is MCP?

The Model Context Protocol is a standardized way for AI applications to:

- **Discover and invoke tools** - Execute functions and commands
- **Access resources** - Read files, query databases, fetch web content  
- **Use prompts and templates** - Generate structured responses
- **Manage context** - Maintain state across interactions

Think of MCP as a universal adapter that allows AI models to interact with any system through a consistent, well-defined interface.

## What is PMCP?

PMCP (Pragmatic Model Context Protocol) is a high-performance Rust implementation that:

- **Maintains 100% TypeScript SDK compatibility** - Drop-in replacement for existing applications
- **Leverages Rust's type system** - Catch protocol errors at compile time
- **Delivers superior performance** - 10x faster than TypeScript implementations  
- **Follows Toyota Way quality standards** - Zero tolerance for defects
- **Provides comprehensive tooling** - Everything you need for production deployment

## Key Features

### 🚀 **Performance**
- **Zero-cost abstractions** - Pay only for what you use
- **Async-first design** - Handle thousands of concurrent connections
- **Memory efficient** - Minimal allocation overhead
- **SIMD optimizations** - Vectorized protocol parsing

### 🔒 **Type Safety**  
- **Compile-time protocol validation** - Catch errors before deployment
- **Rich type system** - Express complex protocol constraints
- **Memory safety** - No segfaults, no data races
- **Resource management** - Automatic cleanup and lifecycle management

### 🔄 **Compatibility**
- **TypeScript SDK parity** - Identical protocol behavior
- **Cross-platform support** - Linux, macOS, Windows, WebAssembly
- **Multiple transports** - WebSocket, HTTP, Streamable HTTP, SSE
- **Version compatibility** - Support for all MCP protocol versions

### 🏭 **Production Ready**
- **Comprehensive testing** - 74%+ coverage, property tests, integration tests
- **Battle-tested examples** - Real-world usage patterns
- **Monitoring and observability** - Built-in metrics and tracing
- **Security hardened** - OAuth2, rate limiting, input validation

## Architecture Overview

```text
+-------------------+     +-------------------+     +-------------------+
|   MCP Client      |<--->|   Transport       |<--->|   MCP Server      |
|                   |     |   Layer           |     |                   |
|  - Tool calls     |     |  - WebSocket      |     |  - Tool handlers  |
|  - Resource req   |     |  - HTTP           |     |  - Resources      |
|  - Prompt use     |     |  - Streamable     |     |  - Prompts        |
+-------------------+     +-------------------+     +-------------------+
```

PMCP provides implementations for all components:

- **Client Library** - Connect to any MCP server
- **Server Framework** - Build custom MCP servers  
- **Transport Implementations** - WebSocket, HTTP, and more
- **Protocol Utilities** - Serialization, validation, error handling

## Getting Started

The fastest way to experience PMCP is through our examples:

```bash
# Install PMCP
cargo add pmcp

# Run a simple server
cargo run --example 02_server_basic

# Connect with a client  
cargo run --example 01_client_initialize
```

## Real-World Example

Here's a complete MCP server in just a few lines:

```rust
use pmcp::{Server, ToolHandler, RequestHandlerExtra, Result};
use serde_json::{json, Value};
use async_trait::async_trait;

struct Calculator;

#[async_trait]
impl ToolHandler for Calculator {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
        let a = args["a"].as_f64().unwrap_or(0.0);
        let b = args["b"].as_f64().unwrap_or(0.0);
        
        Ok(json!({
            "content": [{
                "type": "text", 
                "text": format!("Result: {}", a + b)
            }],
            "isError": false
        }))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    Server::builder()
        .name("calculator-server")
        .version("1.0.0")
        .tool("add", Calculator)
        .build()?
        .run_stdio()
        .await
}
```

This server:
- ✅ Handles tool calls with full type safety
- ✅ Provides structured responses  
- ✅ Includes comprehensive error handling
- ✅ Works with any MCP client (including TypeScript)

## What's Next?

In the following chapters, you'll learn how to:

1. **Install and configure** PMCP for your environment
2. **Build your first server** with tools, resources, and prompts
3. **Create robust clients** that handle errors gracefully
4. **Implement advanced features** like authentication and middleware
5. **Deploy to production** with confidence and monitoring
6. **Integrate with existing systems** using battle-tested patterns

Let's dive in and start building with PMCP!