# MCP Server on Fermyon Spin

This example demonstrates an environment-agnostic MCP server running on Fermyon Spin, a WebAssembly application platform.

## Key Features

- **Environment-agnostic**: Uses the same `WasmMcpServer` API as other WASI environments
- **Type-safe**: Maintains full MCP type safety
- **Standard WASI**: Compiles to `wasm32-wasip1` target
- **Minimal boilerplate**: Thin wrapper around MCP logic

## Prerequisites

1. Install Rust with wasm32-wasip1 target:
```bash
rustup target add wasm32-wasip1
```

2. Install Fermyon Spin:
```bash
curl -fsSL https://developer.fermyon.com/downloads/install.sh | bash
```

## Building

```bash
# Build the WASM component
spin build

# Or manually:
cargo build --target wasm32-wasip1 --release
```

## Running Locally

```bash
# Start the Spin application
spin up

# The server will be available at http://localhost:3000
```

## Testing

### Using MCP Tester with Scenario Files

The deployment can be tested using the mcp-tester tool with predefined scenarios:

```bash
# Test locally running instance
./target/release/mcp-tester scenario \
  http://localhost:3000 \
  examples/wasm-mcp-server/test-scenarios/calculator-test.yaml

# Test production deployment on Fermyon Cloud
# Replace <your-app-name> with your Fermyon app URL
./target/release/mcp-tester scenario \
  https://<your-app-name>.fermyon.app/ \
  examples/wasm-mcp-server/test-scenarios/calculator-test.yaml

# Quick connectivity test
./target/release/mcp-tester scenario \
  https://<your-app-name>.fermyon.app/ \
  examples/wasm-mcp-server/test-scenarios/minimal-test.json

# Basic calculator operations
./target/release/mcp-tester scenario \
  https://<your-app-name>.fermyon.app/ \
  examples/wasm-mcp-server/test-scenarios/calculator-simple.json
```

#### Expected Results

All test scenarios should pass:
```
╔════════════════════════════════════════════════════════════╗
║              MCP SERVER TESTING TOOL v0.1.0               ║
╚════════════════════════════════════════════════════════════╝

TEST RESULTS
════════════════════════════════════════════════════════════
  ✓ Test Addition - 10 + 5
  ✓ Test Subtraction - 20 - 8
  ✓ Test Multiplication - 4 * 7
  ✓ Test Division - 20 / 4
  ✓ Test Division by Zero (error case)
  ✓ Test Invalid Operation (error case)
  ✓ Test Missing Parameter (error case)

SUMMARY
════════════════════════════════════════════════════════════
| Total Tests | 14    |
| Passed      | 14    |

Overall Status: PASSED
```

### Manual Testing with curl

Test with curl:

```bash
# Initialize the MCP session
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
  }'

# List available tools
curl -X POST http://localhost:3000 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": "2",
    "method": "tools/list",
    "params": {}
  }'

# Call a tool (calculator example)
curl -X POST http://localhost:3000 \
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
  }'
```

## Deployment

Deploy to Fermyon Cloud:

```bash
# Login to Fermyon Cloud
spin login

# Deploy the application
spin deploy
```

## Architecture Comparison

### Fermyon Spin (This Example)
```rust
#[http_component]
fn handle_mcp_request(req: Request) -> Result<impl IntoResponse> {
    let server = WasmMcpServer::builder()
        .tool("add", SimpleTool::new(...))
        .build();
    // Handle request...
}
```

### Cloudflare Workers
```rust
#[event(fetch)]
async fn main(req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    let server = WasmMcpServer::builder()
        .tool("add", SimpleTool::new(...))
        .build();
    // Handle request...
}
```

### Key Insight
The MCP logic (`WasmMcpServer`) remains identical across platforms. Only the HTTP handler changes based on the platform's requirements.

## Advantages of Fermyon Spin

1. **Standard WASI**: Uses `wasm32-wasip1` target (not custom `wasm32-unknown-unknown`)
2. **Component Model**: First-class support for WASI components
3. **Simple Deployment**: Single command deployment with `spin deploy`
4. **Local Development**: Easy local testing with `spin up`
5. **No Proprietary APIs**: Standard WASI interfaces

## Tools Included

- **calculator**: Perform arithmetic operations (add, subtract, multiply, divide)
- **weather**: Get weather information for a location (mock data)
- **system_info**: Get runtime information (reports "Fermyon Spin" environment)

## Deployment URL

After deploying with `spin deploy`, your MCP server will be available at:
🔄 `https://<your-app-name>.fermyon.app/`

### Example Deployment for Testing
You can test the MCP protocol with this example deployment:
🔄 https://mcp-fermyon-spin-3juc7zc4.fermyon.app/

## License

MIT