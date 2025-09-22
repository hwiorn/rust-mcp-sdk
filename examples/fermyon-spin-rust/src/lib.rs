use pmcp::server::wasm_server::{SimpleTool, WasmMcpServer};
use pmcp::types::{ClientRequest, Request as McpRequest, ServerCapabilities};
use serde_json::{json, Value};
use spin_sdk::http::{IntoResponse, Request, Response};
use spin_sdk::http_component;

/// Environment-agnostic MCP server running on Fermyon Spin
/// 
/// This example demonstrates how the same MCP server logic can run
/// on different WASI environments with minimal platform-specific code.
#[http_component]
fn handle_request(req: Request) -> anyhow::Result<impl IntoResponse> {
    // Handle CORS preflight
    if *req.method() == spin_sdk::http::Method::Options {
        // Return empty response for CORS preflight
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

    // Build the environment-agnostic MCP server
    // This is the same builder pattern used in Cloudflare Workers
    let server = WasmMcpServer::builder()
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
        // Add math tools
        .tool(
            "add",
            SimpleTool::new(
                "add",
                "Add two numbers",
                |args: Value| {
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
                    
                    Ok(json!({
                        "result": a + b,
                        "operation": "addition"
                    }))
                }
            ).with_schema(json!({
                "type": "object",
                "properties": {
                    "a": {
                        "type": "number",
                        "description": "First number to add"
                    },
                    "b": {
                        "type": "number",
                        "description": "Second number to add"
                    }
                },
                "required": ["a", "b"]
            }))
        )
        // Add string manipulation tool
        .tool(
            "reverse",
            SimpleTool::new(
                "reverse",
                "Reverse a string",
                |args: Value| {
                    let text = args.get("text")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| pmcp::Error::protocol(
                            pmcp::ErrorCode::INVALID_PARAMS,
                            "parameter 'text' is required"
                        ))?;
                    
                    Ok(json!({
                        "original": text,
                        "reversed": text.chars().rev().collect::<String>()
                    }))
                }
            ).with_schema(json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Text string to reverse"
                    }
                },
                "required": ["text"]
            }))
        )
        // Add environment info tool
        .tool(
            "environment",
            SimpleTool::new(
                "environment",
                "Get runtime environment information",
                |_args: Value| {
                    Ok(json!({
                        "runtime": "Fermyon Spin",
                        "wasi_version": "preview2",
                        "sdk": "pmcp",
                        "architecture": "wasm32-wasi",
                        "message": "Environment-agnostic MCP server running in Fermyon Spin!"
                    }))
                }
            ).with_schema(json!({
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }))
        )
        .build();

    // Parse the JSON-RPC request
    let request_value: Value = serde_json::from_str(body)?;
    
    // Extract request components
    let id = request_value.get("id")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or(pmcp::types::RequestId::String("0".to_string()));
    
    let method = request_value.get("method")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing field `method`"))?;
    
    let params = request_value.get("params")
        .cloned()
        .unwrap_or(Value::Null);
    
    // Convert method and params to ClientRequest
    let client_request = match method {
        "initialize" => {
            let params: pmcp::types::InitializeParams = serde_json::from_value(params)?;
            ClientRequest::Initialize(params)
        },
        "tools/list" => {
            let params: pmcp::types::ListToolsParams = serde_json::from_value(params)?;
            ClientRequest::ListTools(params)
        },
        "tools/call" => {
            let params: pmcp::types::CallToolParams = serde_json::from_value(params)?;
            ClientRequest::CallTool(params)
        },
        "resources/list" => {
            let params: pmcp::types::ListResourcesParams = serde_json::from_value(params)?;
            ClientRequest::ListResources(params)
        },
        "resources/read" => {
            let params: pmcp::types::ReadResourceParams = serde_json::from_value(params)?;
            ClientRequest::ReadResource(params)
        },
        "prompts/list" => {
            let params: pmcp::types::ListPromptsParams = serde_json::from_value(params)?;
            ClientRequest::ListPrompts(params)
        },
        "prompts/get" => {
            let params: pmcp::types::GetPromptParams = serde_json::from_value(params)?;
            ClientRequest::GetPrompt(params)
        },
        _ => {
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
    
    // Handle the request synchronously (using futures::executor)
    let response = futures::executor::block_on(async {
        server.handle_request(
            id,
            McpRequest::Client(Box::new(client_request))
        ).await
    });
    
    // Serialize response
    let response_json = serde_json::to_string(&response)?;
    
    // Return HTTP response with proper headers
    let mut response = Response::new(200, response_json);
    response.set_header("content-type", "application/json");
    response.set_header("access-control-allow-origin", "*");
    Ok(response)
}