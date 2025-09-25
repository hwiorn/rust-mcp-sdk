use pmcp::server::wasm_server::{WasmMcpServer, SimpleTool};
use pmcp::types::{ClientRequest, Request as McpRequest, ServerCapabilities};
use serde_json::{json, Value};
use worker::*;

#[event(fetch)]
async fn main(mut req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    // Set panic hook for better error messages
    console_error_panic_hook::set_once();
    
    // Log the request
    console_log!("Received: {} {}", req.method(), req.path());
    
    // Handle CORS preflight
    if req.method() == Method::Options {
        let mut headers = Headers::new();
        headers.set("Access-Control-Allow-Origin", "*")?;
        headers.set("Access-Control-Allow-Methods", "GET, POST, OPTIONS")?;
        headers.set("Access-Control-Allow-Headers", "Content-Type")?;
        return Ok(Response::empty()?.with_headers(headers));
    }
    
    // Handle GET requests with server info
    if req.method() == Method::Get {
        let info = json!({
            "name": "cloudflare-mcp-worker",
            "version": "1.0.0",
            "protocol_version": "2024-11-05",
            "description": "MCP server running on Cloudflare Workers",
            "capabilities": {
                "tools": true,
                "resources": false,
                "prompts": false
            }
        });
        
        let mut headers = Headers::new();
        headers.set("Content-Type", "application/json")?;
        headers.set("Access-Control-Allow-Origin", "*")?;
        
        return Ok(Response::ok(serde_json::to_string_pretty(&info)?)?.with_headers(headers));
    }
    
    // Only handle POST requests for MCP protocol
    if req.method() != Method::Post {
        return Response::error("Only GET and POST methods are supported", 405);
    }
    
    // Get request body
    let body = match req.text().await {
        Ok(text) => text,
        Err(e) => {
            console_error!("Failed to read body: {}", e);
            return Response::error("Failed to read request body", 400);
        }
    };
    
    // Create MCP server with tools using the new environment-agnostic API
    let server = WasmMcpServer::builder()
        .name("cloudflare-mcp-worker")
        .version("1.0.0")
        .capabilities(ServerCapabilities {
            tools: Some(Default::default()),
            resources: None,
            prompts: None,
            logging: None,
            experimental: None,
            completions: None,
            sampling: None,
        })
    
        // Add calculator tool with proper type safety
        .tool(
            "calculator",
            SimpleTool::new(
                "calculator",
                "Perform arithmetic calculations",
                |args: Value| {
                    let operation = args.get("operation")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| pmcp::Error::protocol(
                            pmcp::ErrorCode::INVALID_PARAMS,
                            "operation is required"
                        ))?;
                    
                    let a = args.get("a")
                        .and_then(|v| v.as_f64())
                        .ok_or_else(|| pmcp::Error::protocol(
                            pmcp::ErrorCode::INVALID_PARAMS,
                            "parameter 'a' is required"
                        ))?;
                    
                    let b = args.get("b")
                        .and_then(|v| v.as_f64())
                        .ok_or_else(|| pmcp::Error::protocol(
                            pmcp::ErrorCode::INVALID_PARAMS,
                            "parameter 'b' is required"
                        ))?;
                    
                    let result = match operation {
                        "add" => a + b,
                        "subtract" => a - b,
                        "multiply" => a * b,
                        "divide" => {
                            if b == 0.0 {
                                return Err(pmcp::Error::protocol(
                                    pmcp::ErrorCode::INVALID_PARAMS,
                                    "Division by zero"
                                ));
                            }
                            a / b
                        }
                        _ => return Err(pmcp::Error::protocol(
                            pmcp::ErrorCode::INVALID_PARAMS,
                            &format!("Unknown operation: {}", operation)
                        ))
                    };
                    
                    Ok(json!({
                        "operation": operation,
                        "a": a,
                        "b": b,
                        "result": result
                    }))
                }
            ).with_schema(json!({
                "type": "object",
                "properties": {
                    "operation": {
                        "type": "string",
                        "enum": ["add", "subtract", "multiply", "divide"],
                        "description": "The arithmetic operation to perform"
                    },
                    "a": {
                        "type": "number",
                        "description": "First operand"
                    },
                    "b": {
                        "type": "number",
                        "description": "Second operand"
                    }
                },
                "required": ["operation", "a", "b"]
            }))
        )
    
        // Add weather tool
        .tool(
            "weather",
            SimpleTool::new(
                "weather",
                "Get weather information",
                |args: Value| {
                    let location = args.get("location")
                        .and_then(|v| v.as_str())
                        .unwrap_or("San Francisco");
                    
                    Ok(json!({
                        "location": location,
                        "temperature": "72°F",
                        "conditions": "Sunny",
                        "humidity": "45%"
                    }))
                }
            ).with_schema(json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "Location to get weather for"
                    }
                },
                "required": []
            }))
        )
    
        // Add system info tool
        .tool(
            "system_info",
            SimpleTool::new(
                "system_info",
                "Get system information",
                |_args: Value| {
                    Ok(json!({
                        "runtime": "Cloudflare Workers",
                        "sdk": "pmcp",
                        "version": env!("CARGO_PKG_VERSION"),
                        "architecture": "wasm32-unknown-unknown",
                        "message": "Environment-agnostic MCP server running in Cloudflare Workers!"
                    }))
                }
            ).with_schema(json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }))
        )
        .build();
    
    // Process MCP request using the typed server
    // Parse the JSON-RPC request
    let request_value: Value = match serde_json::from_str(&body) {
        Ok(val) => val,
        Err(e) => {
            console_error!("Failed to parse request: {}", e);
            return Response::error("Invalid JSON-RPC request", 400);
        }
    };

    // Check if this is a notification (no id field means it's a notification)
    let maybe_id = request_value.get("id");

    let method = request_value.get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let params = request_value.get("params")
        .cloned()
        .unwrap_or(Value::Null);

    // Handle notifications - they don't require a response
    if maybe_id.is_none() {
        // This is a notification
        match method {
            "notifications/initialized" => {
                // Client is telling us it's initialized - no action needed
                console_log!("Received initialized notification");
                // Return an empty successful response (notifications don't get responses in JSON-RPC)
                return Response::empty();
            },
            "notifications/cancelled" => {
                console_log!("Received cancellation notification");
                return Response::empty();
            },
            _ => {
                console_log!("Received unknown notification: {}", method);
                return Response::empty();
            }
        }
    }

    // Extract request ID for regular requests
    let id = maybe_id
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or(pmcp::types::RequestId::String("0".to_string()));

    // Construct ClientRequest based on method
    let client_request = match method {
        "initialize" => {
            // Handle initialize params with optional capabilities field
            let mut init_params = params.clone();
            if init_params.is_object() && !init_params.get("capabilities").is_some() {
                // Add empty capabilities if not present (for compatibility with mcp-tester)
                if let Some(obj) = init_params.as_object_mut() {
                    obj.insert("capabilities".to_string(), json!({}));
                }
            }

            let init_params: pmcp::types::InitializeParams = match serde_json::from_value(init_params) {
                Ok(p) => p,
                Err(e) => {
                    console_error!("Failed to parse initialize params: {}", e);
                    return Response::error("Invalid initialize params", 400);
                }
            };
            ClientRequest::Initialize(init_params)
        },
        "tools/list" => {
            ClientRequest::ListTools(pmcp::types::ListToolsParams { cursor: None })
        },
        "tools/call" => {
            let call_params: pmcp::types::CallToolParams = match serde_json::from_value(params) {
                Ok(p) => p,
                Err(e) => {
                    console_error!("Failed to parse tool call params: {}", e);
                    return Response::error("Invalid tool call params", 400);
                }
            };
            ClientRequest::CallTool(call_params)
        },
        _ => {
            console_error!("Unknown method: {}", method);
            // Return proper JSON-RPC error response for unknown methods
            let error_response = json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": format!("Method not found: {}", method)
                }
            });

            return Response::ok(&error_response.to_string())
                .and_then(|mut res| {
                    res.headers_mut().set("Content-Type", "application/json")?;
                    Ok(res)
                })
        }
    };

    // Handle the request with the environment-agnostic server
    let response = server.handle_request(
        id,
        McpRequest::Client(Box::new(client_request))
    ).await;
    
    match serde_json::to_string(&response) {
        Ok(response_json) => {
            console_log!("Response: {}", response_json);
            let mut headers = Headers::new();
            headers.set("Content-Type", "application/json")?;
            headers.set("Access-Control-Allow-Origin", "*")?;
            Ok(Response::ok(response_json)?.with_headers(headers))
        }
        Err(e) => {
            console_error!("Error serializing response: {:?}", e);
            Response::error(&format!("Error: {}", e), 500)
        }
    }
}