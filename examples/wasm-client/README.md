# MCP WASM Client Example

This example demonstrates a browser-based MCP client built with WebAssembly that supports both HTTP and WebSocket transports.

## Features

- **Dual Transport Support**: Automatically detects and uses the appropriate transport (HTTP or WebSocket) based on the server URL
- **Browser-Based**: Runs entirely in the browser using WebAssembly
- **Interactive UI**: Test MCP tools directly from the web interface
- **Multiple Server Support**: Connect to stateful, stateless, or WebSocket MCP servers

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) with `wasm32-unknown-unknown` target
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) for building the WASM module
- Python 3 (for serving the HTML) or any other HTTP server (e.g., `npm install -g http-server`)

## Building the Example

1. Install the WASM target if you haven't already:
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

2. Navigate to this directory:
   ```bash
   cd examples/wasm-client
   ```

3. Run the build script:
   ```bash
   bash build.sh
   ```
   This will compile the Rust code to WASM and generate the necessary JavaScript bindings.

## Running the Example

1. Start a web server to serve the client (choose one):
   ```bash
   # Using Python
   python3 -m http.server 8000
   
   # Or using npm's http-server
   http-server . -p 8000
   ```

2. Open your browser and navigate to:
   ```
   http://localhost:8000
   ```

3. Start one of the example MCP servers:

   **Option A: Stateless HTTP Server (recommended for serverless/Lambda-style deployments):**
   ```bash
   cargo run --example 23_streamable_http_server_stateless --features streamable-http
   ```
   - Runs on `http://localhost:8081`
   - No session management (perfect for serverless)
   - Simple request/response pattern
   - CORS enabled for browser access

   **Option B: Stateful HTTP Server (traditional server with sessions):**
   ```bash
   cargo run --example 22_streamable_http_server_stateful --features streamable-http
   ```
   - Runs on `http://localhost:3000`
   - Session management with unique IDs
   - Supports SSE for real-time updates
   - CORS enabled for browser access

   **Option C: WebSocket Server:**
   ```bash
   cargo run --example 13_websocket_transport --features websocket
   ```
   - Runs on `ws://localhost:8080`
   - Persistent connection
   - Bidirectional communication

4. In the browser interface:
   - The appropriate server URL should already be selected
   - Click "Connect" to establish connection
   - Once connected, you'll see available tools
   - Test the tools by entering JSON arguments and clicking "Call Tool"

## Transport Selection

The client automatically selects the transport based on the URL scheme:
- `http://` or `https://` → HTTP transport (for stateless/stateful servers)
- `ws://` or `wss://` → WebSocket transport

### Which Transport Should I Use?

| Transport | Best For | Pros | Cons |
|-----------|----------|------|------|
| **Stateless HTTP** | Serverless (Lambda, Vercel) | No session overhead, scales horizontally, simple deployment | No real-time updates, higher latency |
| **Stateful HTTP** | Traditional servers | Session management, SSE support, moderate complexity | Requires sticky sessions for scaling |
| **WebSocket** | Real-time applications | Lowest latency, bidirectional, persistent connection | More complex deployment, harder to scale |

## Example Tool Calls

Once connected, you can test tools with JSON arguments. The available tools depend on which example server you're running:

**For the echo tool:**
```json
{"message": "Hello from WASM!"}
```

**For the add tool:**
```json
{"a": 5, "b": 3}
```

**For the get_weather tool:**
```json
{"city": "San Francisco"}
```

## Using with Custom Servers

This WASM client can connect to any MCP server that supports HTTP or WebSocket transport. Simply enter your server's URL in the connection field.

### Deployment Options

1. **Local Development**: Use with any of the SDK example servers
2. **Custom HTTP Servers**: Any MCP server with proper CORS headers
3. **Cloud Services**: Deploy to services that support HTTP/WebSocket
   - AWS Lambda with API Gateway
   - Google Cloud Run
   - Azure Functions
   - Vercel/Netlify Functions
4. **WebSocket Servers**: Any WebSocket-based MCP server

### CORS Configuration

For the browser to connect to your MCP server, the server must send appropriate CORS headers:
- `Access-Control-Allow-Origin: *` (or specific origin like `http://localhost:8000`)
- `Access-Control-Allow-Methods: POST, OPTIONS`
- `Access-Control-Allow-Headers: Content-Type, mcp-session-id, mcp-protocol-version`

The example servers (22, 23) already include proper CORS configuration.

## Using the WASM Client Programmatically

You can also use the WASM client in your own JavaScript/TypeScript applications:

```javascript
import init, { WasmClient } from './pkg/mcp_wasm_client.js';

async function connectToMCP() {
    // Initialize the WASM module
    await init();
    
    // Create a new client
    const client = new WasmClient();
    
    // Connect to a server (auto-detects transport type)
    await client.connect("http://localhost:8081");  // HTTP
    // or
    // await client.connect("ws://localhost:8080");  // WebSocket
    
    // List available tools
    const tools = await client.list_tools();
    console.log("Available tools:", tools);
    
    // Call a tool
    const result = await client.call_tool("echo", {
        message: "Hello from JavaScript!"
    });
    console.log("Tool result:", result);
}
```

## Architecture

The WASM client consists of:
- **Rust WASM Module** (`src/lib.rs`): Core MCP client logic compiled to WebAssembly
- **HTTP Transport** (`src/shared/wasm_http.rs`): Browser Fetch API-based transport for HTTP servers
- **WebSocket Transport** (`src/shared/wasm_websocket.rs`): Browser WebSocket API-based transport
- **HTML Interface** (`index.html`): Interactive UI for testing the client
- **Auto-detection**: Automatically selects transport based on URL scheme

### Integration with Examples 22, 23, 24

This WASM client works seamlessly with the streamable HTTP server examples:

- **Example 22 (Stateful)**: Full session management with SSE support
- **Example 23 (Stateless)**: Optimized for serverless deployments without sessions
- **Example 24 (Client)**: Demonstrates native Rust client with same transport

All three examples share the same transport protocol, allowing this WASM client to connect to any of them.

## Troubleshooting

**Build Errors:**
- Ensure you have the `wasm32-unknown-unknown` target installed
- Make sure `wasm-pack` is up to date: `cargo install wasm-pack --force`
- Check that all dependencies in `Cargo.toml` have WASM-compatible features

**Connection Errors:**
- Verify the server is running on the expected port
- Check browser console (F12) for detailed error messages
- Ensure the server supports the transport type you're using
- For HTTP servers, verify CORS headers are properly configured

**"Failed to parse response" Error:**
- This usually means the server response isn't in the expected JSON-RPC format
- Check the server logs for any errors
- Verify the server is an MCP server (not a regular HTTP server)
