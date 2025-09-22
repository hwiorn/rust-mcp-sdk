//! MCP Server for Cloudflare Workers
//! 
//! This example demonstrates the new transport-agnostic architecture
//! by implementing a minimal MCP server that compiles to WASM and runs
//! on Cloudflare Workers.

mod types;

use serde_json::{json, Value};
use std::collections::HashMap;
use worker::{console_error, console_log, Context, Env, Method, Request, Response, Result as WorkerResult};

/// Minimal protocol handler for WASM
/// This demonstrates the core concept from our refactoring:
/// A protocol handler that is completely independent of transport
pub struct MCPProtocolHandler {
    server_info: types::Implementation,
    capabilities: types::ServerCapabilities,
    tools: HashMap<String, Box<dyn Fn(Value) -> types::Result<Value>>>,
    initialized: bool,
}

impl MCPProtocolHandler {
    pub fn new(name: String, version: String) -> Self {
        let mut tools = HashMap::new();
        
        // Weather tool
        tools.insert("weather".to_string(), Box::new(|args: Value| {
            let location = args.get("location")
                .and_then(|v| v.as_str())
                .unwrap_or("San Francisco");
            
            Ok(json!({
                "location": location,
                "temperature": "72°F",
                "conditions": "Sunny",
                "humidity": "45%",
                "forecast": "Clear skies expected throughout the day"
            }))
        }) as Box<dyn Fn(Value) -> types::Result<Value>>);
        
        // Calculator tool
        tools.insert("calculator".to_string(), Box::new(|args: Value| {
            let operation = args.get("operation")
                .and_then(|v| v.as_str())
                .ok_or_else(|| types::Error::Validation("operation is required".to_string()))?;
            
            let a = args.get("a")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| types::Error::Validation("parameter 'a' is required".to_string()))?;
            
            let b = args.get("b")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| types::Error::Validation("parameter 'b' is required".to_string()))?;
            
            let result = match operation {
                "add" => a + b,
                "subtract" => a - b,
                "multiply" => a * b,
                "divide" => {
                    if b == 0.0 {
                        return Err(types::Error::Validation("Division by zero".to_string()));
                    }
                    a / b
                },
                _ => return Err(types::Error::Validation(format!("Unknown operation: {}", operation))),
            };
            
            Ok(json!({
                "operation": operation,
                "a": a,
                "b": b,
                "result": result
            }))
        }) as Box<dyn Fn(Value) -> types::Result<Value>>);
        
        Self {
            server_info: types::Implementation { name, version },
            capabilities: types::ServerCapabilities {
                tools: Some(HashMap::from([("available".to_string(), json!(true))])),
                logging: Some(HashMap::from([("supported".to_string(), json!(true))])),
                ..Default::default()
            },
            tools,
            initialized: false,
        }
    }
    
    /// Handle a JSON-RPC request - this is transport-independent!
    pub fn handle_request(&mut self, request: types::JSONRPCRequest) -> types::JSONRPCResponse {
        match request.method.as_str() {
            "initialize" => {
                self.initialized = true;
                types::JSONRPCResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: types::ResponseResult::Success {
                        result: serde_json::to_value(types::InitializeResult {
                            protocol_version: "2024-11-05".to_string(),
                            capabilities: self.capabilities.clone(),
                            server_info: self.server_info.clone(),
                        }).unwrap(),
                    },
                }
            },
            "tools/list" => {
                if !self.initialized {
                    return types::Error::Internal("Not initialized".to_string()).to_jsonrpc(request.id);
                }
                
                let tools: Vec<types::Tool> = self.tools.keys().map(|name| types::Tool {
                    name: name.clone(),
                    description: Some(format!("{} tool", name)),
                    input_schema: json!({
                        "type": "object",
                        "properties": {}
                    }),
                }).collect();
                
                types::JSONRPCResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: types::ResponseResult::Success {
                        result: serde_json::to_value(types::ListToolsResult { tools }).unwrap(),
                    },
                }
            },
            "tools/call" => {
                if !self.initialized {
                    return types::Error::Internal("Not initialized".to_string()).to_jsonrpc(request.id);
                }
                
                let params: types::CallToolParams = match serde_json::from_value(request.params) {
                    Ok(p) => p,
                    Err(e) => return types::Error::Validation(format!("Invalid params: {}", e)).to_jsonrpc(request.id),
                };
                
                match self.tools.get(&params.name) {
                    Some(tool) => {
                        match tool(params.arguments) {
                            Ok(result) => types::JSONRPCResponse {
                                jsonrpc: "2.0".to_string(),
                                id: request.id,
                                result: types::ResponseResult::Success {
                                    result: serde_json::to_value(types::CallToolResult {
                                        content: vec![types::Content::Text {
                                            text: result.to_string(),
                                        }],
                                    }).unwrap(),
                                },
                            },
                            Err(e) => e.to_jsonrpc(request.id),
                        }
                    },
                    None => types::Error::MethodNotFound(format!("Tool not found: {}", params.name)).to_jsonrpc(request.id),
                }
            },
            _ => types::Error::MethodNotFound(format!("Method not found: {}", request.method)).to_jsonrpc(request.id),
        }
    }
    
    /// Handle a JSON-RPC notification - this is also transport-independent!
    pub fn handle_notification(&mut self, _notification: types::JSONRPCNotification) -> types::Result<()> {
        // Notifications don't require responses
        // Could handle progress notifications, logging, etc.
        Ok(())
    }
}

/// Cloudflare Worker HTTP Adapter
/// This is the transport-specific part that bridges HTTP to our protocol handler
pub struct CloudflareWorkerAdapter {
    handler: MCPProtocolHandler,
}

impl CloudflareWorkerAdapter {
    pub fn new() -> Self {
        Self {
            handler: MCPProtocolHandler::new(
                "cloudflare-worker-mcp".to_string(),
                "1.0.0".to_string(),
            ),
        }
    }
    
    /// Convert HTTP request to protocol handler call and back
    pub async fn handle_http_request(&mut self, mut req: Request) -> WorkerResult<Response> {
        // Parse the incoming request body
        let body = req.text().await?;
        
        // Try to parse as JSON-RPC request
        if let Ok(request) = serde_json::from_str::<types::JSONRPCRequest>(&body) {
            // Handle request through protocol handler
            let response = self.handler.handle_request(request);
            
            // Serialize response
            let response_body = serde_json::to_string(&response)
                .map_err(|e| worker::Error::RustError(format!("Failed to serialize: {}", e)))?;
            
            // Return HTTP response
            {
                let mut resp = Response::ok(response_body)?;
                resp.headers_mut().set("Content-Type", "application/json")?;
                Ok(resp)
            }
        } else if let Ok(notification) = serde_json::from_str::<types::JSONRPCNotification>(&body) {
            // Handle notification
            self.handler.handle_notification(notification)
                .map_err(|e| worker::Error::RustError(format!("Notification failed: {:?}", e)))?;
            
            // Notifications don't get responses
            Response::empty()
        } else {
            Response::error("Invalid JSON-RPC message", 400)
        }
    }
}

// Cloudflare Worker Entry Point
#[worker::event(fetch)]
pub async fn main(req: Request, _env: Env, _ctx: Context) -> WorkerResult<Response> {
    // Set up panic hook for better error messages
    console_error_panic_hook::set_once();
    
    // Log request for debugging
    console_log!("Received request: {} {}", req.method(), req.url()?.to_string());
    
    // Handle CORS preflight
    if req.method() == Method::Options {
        let mut resp = Response::empty()?;
        resp.headers_mut().set("Access-Control-Allow-Origin", "*")?;
        resp.headers_mut().set("Access-Control-Allow-Methods", "GET, POST, OPTIONS")?;
        resp.headers_mut().set("Access-Control-Allow-Headers", "Content-Type")?;
        return Ok(resp);
    }
    
    // Only handle POST requests for JSON-RPC
    if req.method() != Method::Post {
        return Response::error("Only POST method is supported", 405);
    }
    
    // Create adapter (in production, this could be cached)
    let mut adapter = CloudflareWorkerAdapter::new();
    
    // Handle the request
    match adapter.handle_http_request(req).await {
        Ok(mut response) => {
            // Add CORS headers
            response.headers_mut().set("Access-Control-Allow-Origin", "*")?;
            Ok(response)
        },
        Err(e) => {
            console_error!("Request handling failed: {:?}", e);
            Response::error("Server error", 500)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_protocol_handler_initialization() {
        let mut handler = MCPProtocolHandler::new(
            "test".to_string(),
            "1.0.0".to_string()
        );
        
        let request = types::JSONRPCRequest {
            jsonrpc: "2.0".to_string(),
            id: types::RequestId::Number(1),
            method: "initialize".to_string(),
            params: serde_json::to_value(types::InitializeParams {
                protocol_version: "2024-11-05".to_string(),
                capabilities: types::ClientCapabilities::default(),
                client_info: types::Implementation {
                    name: "test-client".to_string(),
                    version: "1.0.0".to_string(),
                },
            }).unwrap(),
        };
        
        let response = handler.handle_request(request);
        
        match response.result {
            types::ResponseResult::Success { result } => {
                let init_result: types::InitializeResult = serde_json::from_value(result).unwrap();
                assert_eq!(init_result.server_info.name, "test");
                assert_eq!(init_result.protocol_version, "2024-11-05");
            },
            _ => panic!("Expected success response"),
        }
    }
    
    #[test]
    fn test_calculator_tool() {
        let mut handler = MCPProtocolHandler::new(
            "test".to_string(),
            "1.0.0".to_string()
        );
        
        // Initialize first
        handler.initialized = true;
        
        let request = types::JSONRPCRequest {
            jsonrpc: "2.0".to_string(),
            id: types::RequestId::Number(2),
            method: "tools/call".to_string(),
            params: serde_json::to_value(types::CallToolParams {
                name: "calculator".to_string(),
                arguments: json!({
                    "operation": "multiply",
                    "a": 6,
                    "b": 7
                }),
            }).unwrap(),
        };
        
        let response = handler.handle_request(request);
        
        match response.result {
            types::ResponseResult::Success { result } => {
                let tool_result: types::CallToolResult = serde_json::from_value(result).unwrap();
                assert_eq!(tool_result.content.len(), 1);
            },
            _ => panic!("Expected success response"),
        }
    }
}