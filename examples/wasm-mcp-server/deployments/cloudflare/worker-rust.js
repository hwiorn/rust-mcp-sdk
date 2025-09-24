// Cloudflare Workers entry point for WASM-based MCP server
import init, { fetch as wasmFetch } from './pkg/mcp_cloudflare_worker.js';
import wasmModule from './pkg/mcp_cloudflare_worker_bg.wasm';

// Initialize the WASM module once
let wasmInitialized = false;
async function ensureWasmInit() {
  if (!wasmInitialized) {
    await init(wasmModule);
    wasmInitialized = true;
  }
}

export default {
  async fetch(request, env, ctx) {
    // Initialize WASM if needed
    await ensureWasmInit();
    
    try {
      // Call the Rust fetch handler
      return await wasmFetch(request, env, ctx);
    } catch (error) {
      console.error('WASM fetch error:', error);
      console.error('Stack:', error.stack);
      
      // Return error response
      return new Response(JSON.stringify({
        jsonrpc: "2.0",
        id: null,
        error: {
          code: -32603,
          message: "Internal error",
          data: error.toString()
        }
      }), {
        status: 500,
        headers: {
          "Content-Type": "application/json",
          "Access-Control-Allow-Origin": "*"
        }
      });
    }
  }
};