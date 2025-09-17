// Cloudflare Worker entry point
// This is a simple wrapper that loads the WASM module

export default {
    async fetch(request, env, ctx) {
        try {
            // For now, just return a simple response showing the server is working
            // In a real implementation, you would load and use the WASM module here
            
            const url = new URL(request.url);
            
            // Handle CORS preflight
            if (request.method === 'OPTIONS') {
                return new Response(null, {
                    headers: {
                        'Access-Control-Allow-Origin': '*',
                        'Access-Control-Allow-Methods': 'POST, OPTIONS',
                        'Access-Control-Allow-Headers': 'Content-Type',
                    }
                });
            }
            
            // Only accept POST requests for MCP
            if (request.method !== 'POST') {
                return new Response(JSON.stringify({
                    jsonrpc: "2.0",
                    id: null,
                    error: {
                        code: -32600,
                        message: "Invalid Request - Only POST method is supported"
                    }
                }), {
                    status: 400,
                    headers: {
                        'Content-Type': 'application/json',
                        'Access-Control-Allow-Origin': '*'
                    }
                });
            }
            
            // Parse the request body
            let body;
            try {
                body = await request.json();
            } catch (e) {
                return new Response(JSON.stringify({
                    jsonrpc: "2.0",
                    id: null,
                    error: {
                        code: -32700,
                        message: "Parse error"
                    }
                }), {
                    status: 400,
                    headers: {
                        'Content-Type': 'application/json',
                        'Access-Control-Allow-Origin': '*'
                    }
                });
            }
            
            // Handle initialize request
            if (body.method === 'initialize') {
                return new Response(JSON.stringify({
                    jsonrpc: "2.0",
                    id: body.id,
                    result: {
                        protocolVersion: "2024-11-05",
                        capabilities: {
                            tools: {},
                            prompts: {},
                            resources: {}
                        },
                        serverInfo: {
                            name: "cloudflare-worker-mcp",
                            version: "0.1.0"
                        }
                    }
                }), {
                    headers: {
                        'Content-Type': 'application/json',
                        'Access-Control-Allow-Origin': '*'
                    }
                });
            }
            
            // Default response for other methods
            return new Response(JSON.stringify({
                jsonrpc: "2.0",
                id: body.id || null,
                error: {
                    code: -32601,
                    message: "Method not found"
                }
            }), {
                status: 404,
                headers: {
                    'Content-Type': 'application/json',
                    'Access-Control-Allow-Origin': '*'
                }
            });
            
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
                    'Content-Type': 'application/json',
                    'Access-Control-Allow-Origin': '*'
                }
            });
        }
    }
};