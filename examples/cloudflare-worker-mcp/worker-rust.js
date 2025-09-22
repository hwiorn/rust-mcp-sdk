// Wrapper for Rust WASM Worker
import wasm from './build/cloudflare_worker_mcp_sdk_bg.wasm';
import * as exports from './build/cloudflare_worker_mcp_sdk_bg.js';

// Re-export the fetch handler from the WASM module
export default {
  fetch: exports.fetch || exports.main || async (request, env, ctx) => {
    return new Response('Worker not properly initialized', { status: 500 });
  }
};