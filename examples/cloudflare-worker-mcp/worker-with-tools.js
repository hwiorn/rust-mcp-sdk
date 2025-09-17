// Cloudflare Worker with Tools Support
// This demonstrates how easy it is to add tools to our MCP server

export default {
    async fetch(request, env, ctx) {
        try {
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
            
            // Only accept POST requests
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
            
            // Route to appropriate handler
            switch (body.method) {
                case 'initialize':
                    return handleInitialize(body);
                case 'tools/list':
                    return handleListTools(body);
                case 'tools/call':
                    return handleCallTool(body);
                default:
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
            }
            
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

// Handle initialize request
function handleInitialize(body) {
    return new Response(JSON.stringify({
        jsonrpc: "2.0",
        id: body.id,
        result: {
            protocolVersion: "2024-11-05",
            capabilities: {
                tools: {
                    listChanged: true  // We support tool listing
                },
                prompts: {},
                resources: {}
            },
            serverInfo: {
                name: "cloudflare-worker-mcp",
                version: "0.2.0"
            }
        }
    }), {
        headers: {
            'Content-Type': 'application/json',
            'Access-Control-Allow-Origin': '*'
        }
    });
}

// Handle tools/list request
function handleListTools(body) {
    const tools = [
        {
            name: "calculate",
            description: "Perform basic arithmetic calculations",
            inputSchema: {
                type: "object",
                properties: {
                    operation: {
                        type: "string",
                        enum: ["add", "subtract", "multiply", "divide"],
                        description: "The arithmetic operation to perform"
                    },
                    a: {
                        type: "number",
                        description: "First operand"
                    },
                    b: {
                        type: "number",
                        description: "Second operand"
                    }
                },
                required: ["operation", "a", "b"]
            }
        },
        {
            name: "get_weather",
            description: "Get current weather for a location (mock data)",
            inputSchema: {
                type: "object",
                properties: {
                    location: {
                        type: "string",
                        description: "City name or location"
                    }
                },
                required: ["location"]
            }
        },
        {
            name: "generate_uuid",
            description: "Generate a random UUID",
            inputSchema: {
                type: "object",
                properties: {}
            }
        },
        {
            name: "echo",
            description: "Echo back the input message",
            inputSchema: {
                type: "object",
                properties: {
                    message: {
                        type: "string",
                        description: "Message to echo"
                    }
                },
                required: ["message"]
            }
        }
    ];
    
    return new Response(JSON.stringify({
        jsonrpc: "2.0",
        id: body.id,
        result: {
            tools: tools
        }
    }), {
        headers: {
            'Content-Type': 'application/json',
            'Access-Control-Allow-Origin': '*'
        }
    });
}

// Handle tools/call request
function handleCallTool(body) {
    const { name, arguments: args } = body.params;
    
    let result;
    let isError = false;
    
    switch (name) {
        case "calculate":
            result = executeCalculate(args);
            break;
        case "get_weather":
            result = executeGetWeather(args);
            break;
        case "generate_uuid":
            result = executeGenerateUuid();
            break;
        case "echo":
            result = executeEcho(args);
            break;
        default:
            isError = true;
            result = {
                code: -32602,
                message: `Unknown tool: ${name}`
            };
    }
    
    if (isError) {
        return new Response(JSON.stringify({
            jsonrpc: "2.0",
            id: body.id,
            error: result
        }), {
            status: 400,
            headers: {
                'Content-Type': 'application/json',
                'Access-Control-Allow-Origin': '*'
            }
        });
    }
    
    return new Response(JSON.stringify({
        jsonrpc: "2.0",
        id: body.id,
        result: {
            content: [
                {
                    type: "text",
                    text: JSON.stringify(result, null, 2)
                }
            ],
            isError: false
        }
    }), {
        headers: {
            'Content-Type': 'application/json',
            'Access-Control-Allow-Origin': '*'
        }
    });
}

// Tool implementations
function executeCalculate(args) {
    const { operation, a, b } = args;
    let result;
    
    switch (operation) {
        case "add":
            result = a + b;
            break;
        case "subtract":
            result = a - b;
            break;
        case "multiply":
            result = a * b;
            break;
        case "divide":
            if (b === 0) {
                return { error: "Division by zero" };
            }
            result = a / b;
            break;
        default:
            return { error: "Unknown operation" };
    }
    
    return {
        operation: operation,
        a: a,
        b: b,
        result: result
    };
}

function executeGetWeather(args) {
    const { location } = args;
    
    // Mock weather data
    const mockWeather = {
        location: location,
        temperature: Math.floor(Math.random() * 30) + 10,
        condition: ["sunny", "cloudy", "rainy", "partly cloudy"][Math.floor(Math.random() * 4)],
        humidity: Math.floor(Math.random() * 50) + 30,
        windSpeed: Math.floor(Math.random() * 20) + 5,
        timestamp: new Date().toISOString()
    };
    
    return mockWeather;
}

function executeGenerateUuid() {
    // Simple UUID v4 generator
    return {
        uuid: 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
            const r = Math.random() * 16 | 0;
            const v = c === 'x' ? r : (r & 0x3 | 0x8);
            return v.toString(16);
        })
    };
}

function executeEcho(args) {
    return {
        message: args.message,
        echoed_at: new Date().toISOString(),
        worker_version: "0.2.0"
    };
}