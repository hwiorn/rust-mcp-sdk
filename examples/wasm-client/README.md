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

   **For Stateless HTTP Server (recommended for first test):**
   ```bash
   cargo run --example 23_streamable_http_server_stateless --features streamable-http
   ```
   The server will run on `http://localhost:8081`

   **For Stateful HTTP Server:**
   ```bash
   cargo run --example 22_streamable_http_server_stateful --features streamable-http
   ```
   The server will run on `http://localhost:3000`

   **For WebSocket Server:**
   ```bash
   cargo run --example 13_websocket_transport --features websocket
   ```
   The server will run on `ws://localhost:8080`

4. In the browser interface:
   - The appropriate server URL should already be selected
   - Click "Connect" to establish connection
   - Once connected, you'll see available tools
   - Test the tools by entering JSON arguments and clicking "Call Tool"

## Transport Selection

The client automatically selects the transport based on the URL scheme:
- `http://` or `https://` → HTTP transport (for stateless/stateful servers)
- `ws://` or `wss://` → WebSocket transport

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

## Architecture

The WASM client consists of:
- **Rust WASM Module** (`src/lib.rs`): Core MCP client logic compiled to WebAssembly
- **HTTP Transport**: Browser Fetch API-based transport for HTTP servers
- **WebSocket Transport**: Browser WebSocket API-based transport
- **HTML Interface** (`index.html`): Interactive UI for testing the client

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
