// Cloudflare Worker entry point
// Import the wasm-pack generated module
import init, { handle_request } from '../build/index.js';
import wasmModule from '../build/index_bg.wasm';

// Track initialization
let initialized = false;

export default {
    async fetch(request, env, ctx) {
        try {
            // Initialize WASM module once
            if (!initialized) {
                await init(wasmModule);
                initialized = true;
            }
            
            // Handle the request using our WASM module
            return await handle_request(request, env);
        } catch (error) {
            console.error('Worker error:', error);
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
                    'Content-Type': 'application/json'
                }
            });
        }
    }
};