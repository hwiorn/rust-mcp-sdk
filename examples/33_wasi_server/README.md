# Example 33: MCP Server as a WASM module

This example demonstrates a cutting-edge approach to deploying MCP servers by compiling the core server logic into a WebAssembly (WASM) module. This allows for ultra-fast, secure, and portable deployments on serverless and edge computing platforms that support the WebAssembly System Interface (WASI).

## Architecture

This example is split into two distinct crates within a workspace:

1.  **`mcp-wasi-server`**:
    *   This is the actual MCP server, compiled to the `wasm32-wasi` target.
    *   It contains all the core MCP logic (tool definitions, request handling) but does **not** bind to any network sockets.
    *   It exposes a single function, `handle_request`, which takes a raw HTTP request body and returns a raw HTTP response body.

2.  **`mcp-wasi-host`**:
    *   This is a native Rust binary that acts as a *host* for the WASM module.
    *   It uses the `wasmtime` crate to load and run `mcp-wasi-server.wasm`.
    *   It uses `hyper` to run a standard HTTP server that listens for requests.
    *   When a request is received, the host passes the request body to the WASM module's `handle_request` function, gets the response back, and sends it to the client.
    *   This host allows for easy local testing and debugging of the WASM server.

This host/guest architecture is the standard for running WASM on the server-side and mirrors how platforms like Cloudflare Workers or Fastly Compute@Edge operate.

## How to Run

1.  **Build the WASM Server:**
    ```bash
    # From within examples/33_wasi_server/mcp-wasi-server
    cargo build --target wasm32-wasi --release
    ```

2.  **Run the Host Application:**
    ```bash
    # From within examples/33_wasi_server/mcp-wasi-host
    cargo run
    ```
    The host will automatically load the `.wasm` file from the target directory.

3.  **Send a Request:**
    ```bash
    curl -X POST http://127.0.0.1:3000 -d '{"jsonrpc":"2.0","method":"tools/list","id":1}'
    ```
