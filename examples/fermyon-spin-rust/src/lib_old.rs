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
fn handle_mcp_request(req: Request) -> anyhow::Result<impl IntoResponse> {
    // Handle CORS preflight
    if *req.method() == spin_sdk::http::Method::Options {
        // Return empty response for CORS preflight
        return Ok(Response::new(200, ()));
    }

    // Only handle POST requests
    if *req.method() != spin_sdk::http::Method::Post {
        return Ok(Response::new(405, "Only POST method is supported"));
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
            )
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
            )
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
            )
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
            return Ok(Response::new(200, serde_json::to_string(&json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": format!("Method not found: {}", method)
                }
            }))?))
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
    
    // Return HTTP response
    Ok(Response::new(200, response_json))
}