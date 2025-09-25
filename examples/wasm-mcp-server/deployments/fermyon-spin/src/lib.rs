use pmcp::server::wasm_server::{SimpleTool, WasmMcpServer};
use pmcp::types::{ClientRequest, Request as McpRequest, ServerCapabilities};
use serde_json::{json, Value};
use spin_sdk::http::{IntoResponse, Request, Response};
use spin_sdk::http_component;

/// Environment-agnostic MCP server running on Fermyon Spin
///
/// This is a thin platform-specific wrapper around the shared MCP server logic.
/// The actual MCP implementation is shared across all WASM platforms.
#[http_component]
fn handle_request(req: Request) -> anyhow::Result<impl IntoResponse> {
    // Handle CORS preflight
    if *req.method() == spin_sdk::http::Method::Options {
        let mut response = Response::new(200, ());
        response.set_header("access-control-allow-origin", "*");
        response.set_header("access-control-allow-methods", "POST, OPTIONS");
        response.set_header("access-control-allow-headers", "Content-Type");
        return Ok(response);
    }

    // Handle GET requests with server info
    if *req.method() == spin_sdk::http::Method::Get {
        let info = json!({
            "name": "fermyon-spin-mcp-server",
            "version": "1.0.0",
            "protocol_version": "2024-11-05",
            "description": "MCP server running on Fermyon Spin",
            "capabilities": {
                "tools": true,
                "resources": false,
                "prompts": false
            }
        });
        let mut response = Response::new(200, serde_json::to_string_pretty(&info)?);
        response.set_header("content-type", "application/json");
        response.set_header("access-control-allow-origin", "*");
        return Ok(response);
    }

    // Only handle POST requests for MCP protocol
    if *req.method() != spin_sdk::http::Method::Post {
        let mut response = Response::new(405, "Only GET and POST methods are supported");
        response.set_header("content-type", "text/plain");
        return Ok(response);
    }

    // Get request body
    let body_bytes = req.body();
    let body = std::str::from_utf8(&body_bytes)?;

    // Parse the JSON-RPC request
    let request_value: Value = serde_json::from_str(body)?;

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
        match method {
            "notifications/initialized" => {
                // Client is telling us it's initialized - return empty 200 OK
                let response = Response::new(200, ());
                return Ok(response);
            },
            "notifications/cancelled" => {
                // Cancellation notification - return empty 200 OK
                let response = Response::new(200, ());
                return Ok(response);
            },
            _ => {
                // Unknown notification - still return empty 200 OK
                let response = Response::new(200, ());
                return Ok(response);
            }
        }
    }

    // Extract request ID for regular requests
    let id = maybe_id
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or(pmcp::types::RequestId::String("0".to_string()));

    // Build the environment-agnostic MCP server
    let server = create_mcp_server();

    // Construct ClientRequest based on method
    let client_request = match method {
        "initialize" => {
            let mut init_params = params.clone();
            if init_params.is_object() && !init_params.get("capabilities").is_some() {
                if let Some(obj) = init_params.as_object_mut() {
                    obj.insert("capabilities".to_string(), json!({}));
                }
            }
            let init_params: pmcp::types::InitializeParams = serde_json::from_value(init_params)?;
            ClientRequest::Initialize(init_params)
        },
        "tools/list" => {
            ClientRequest::ListTools(pmcp::types::ListToolsParams { cursor: None })
        },
        "tools/call" => {
            let call_params: pmcp::types::CallToolParams = serde_json::from_value(params)?;
            ClientRequest::CallTool(call_params)
        },
        _ => {
            // Return proper JSON-RPC error for unknown methods
            let error_response = json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": format!("Method not found: {}", method)
                }
            });
            let mut response = Response::new(200, serde_json::to_string(&error_response)?);
            response.set_header("content-type", "application/json");
            response.set_header("access-control-allow-origin", "*");
            return Ok(response);
        }
    };

    // Handle the request with the environment-agnostic server
    let response = futures::executor::block_on(
        server.handle_request(id, McpRequest::Client(Box::new(client_request)))
    );

    // Return the JSON-RPC response
    let response_json = serde_json::to_string(&response)?;
    let mut http_response = Response::new(200, response_json);
    http_response.set_header("content-type", "application/json");
    http_response.set_header("access-control-allow-origin", "*");

    Ok(http_response)
}

/// Create the shared MCP server with tools
/// This function contains the "write once" MCP logic that's shared across platforms
fn create_mcp_server() -> WasmMcpServer {
    WasmMcpServer::builder()
        .name("fermyon-spin-mcp-server")
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
        // Calculator tool
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
        // Weather tool
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
        // System info tool
        .tool(
            "system_info",
            SimpleTool::new(
                "system_info",
                "Get system information",
                |_args: Value| {
                    Ok(json!({
                        "runtime": "Fermyon Spin",
                        "sdk": "pmcp",
                        "version": env!("CARGO_PKG_VERSION"),
                        "architecture": "wasm32-wasip1",
                        "message": "Environment-agnostic MCP server running in Fermyon Spin!"
                    }))
                }
            ).with_schema(json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }))
        )
        .build()
}