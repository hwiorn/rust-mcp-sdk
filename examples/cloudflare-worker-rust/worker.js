// Simple JavaScript wrapper for Cloudflare Workers
// This avoids the complexity of worker-build

export default {
  async fetch(request, env, ctx) {
    // Handle CORS preflight
    if (request.method === "OPTIONS") {
      return new Response(null, {
        headers: {
          "Access-Control-Allow-Origin": "*",
          "Access-Control-Allow-Methods": "POST, OPTIONS",
          "Access-Control-Allow-Headers": "Content-Type",
        }
      });
    }

    // Only accept POST requests
    if (request.method !== "POST") {
      return new Response("Only POST method supported", { status: 405 });
    }

    // Get request body
    const body = await request.text();
    
    try {
      // Parse as JSON-RPC request
      const jsonRequest = JSON.parse(body);
      
      // Simple MCP server implementation
      let response;
      
      switch (jsonRequest.method) {
        case "initialize":
          response = {
            jsonrpc: "2.0",
            id: jsonRequest.id,
            result: {
              protocolVersion: jsonRequest.params?.protocolVersion || "2024-11-05",
              capabilities: {
                tools: {}
              },
              serverInfo: {
                name: "cloudflare-mcp-worker",
                version: "1.0.0"
              }
            }
          };
          break;
          
        case "tools/list":
          response = {
            jsonrpc: "2.0",
            id: jsonRequest.id,
            result: {
              tools: [
                {
                  name: "calculator",
                  description: "Perform arithmetic calculations"
                },
                {
                  name: "weather",
                  description: "Get weather information"
                },
                {
                  name: "system_info",
                  description: "Get system information"
                }
              ]
            }
          };
          break;
          
        case "tools/call":
          const toolName = jsonRequest.params?.name;
          const args = jsonRequest.params?.arguments || {};
          
          let result;
          if (toolName === "calculator") {
            const op = args.operation || "add";
            const a = args.a || 0;
            const b = args.b || 0;
            let calcResult = 0;
            
            switch (op) {
              case "add": calcResult = a + b; break;
              case "subtract": calcResult = a - b; break;
              case "multiply": calcResult = a * b; break;
              case "divide": calcResult = b !== 0 ? a / b : 0; break;
            }
            
            result = {
              content: [{
                type: "text",
                text: JSON.stringify({
                  operation: op,
                  a: a,
                  b: b,
                  result: calcResult
                }, null, 2)
              }],
              isError: false
            };
          } else if (toolName === "weather") {
            result = {
              content: [{
                type: "text",
                text: JSON.stringify({
                  location: args.location || "San Francisco",
                  temperature: "72°F",
                  conditions: "Sunny",
                  humidity: "45%"
                }, null, 2)
              }],
              isError: false
            };
          } else if (toolName === "system_info") {
            result = {
              content: [{
                type: "text",
                text: JSON.stringify({
                  runtime: "Cloudflare Workers",
                  sdk: "JavaScript (simplified)",
                  architecture: "V8 isolate",
                  message: "MCP server running in Cloudflare Workers!"
                }, null, 2)
              }],
              isError: false
            };
          } else {
            result = {
              content: [{
                type: "text",
                text: `Error: Unknown tool '${toolName}'`
              }],
              isError: true
            };
          }
          
          response = {
            jsonrpc: "2.0",
            id: jsonRequest.id,
            result: result
          };
          break;
          
        default:
          response = {
            jsonrpc: "2.0",
            id: jsonRequest.id,
            error: {
              code: -32601,
              message: `Method not found: ${jsonRequest.method}`
            }
          };
      }
      
      return new Response(JSON.stringify(response), {
        headers: {
          "Content-Type": "application/json",
          "Access-Control-Allow-Origin": "*"
        }
      });
      
    } catch (error) {
      return new Response(JSON.stringify({
        jsonrpc: "2.0",
        id: null,
        error: {
          code: -32700,
          message: "Parse error",
          data: error.message
        }
      }), {
        status: 400,
        headers: {
          "Content-Type": "application/json",
          "Access-Control-Allow-Origin": "*"
        }
      });
    }
  }
};