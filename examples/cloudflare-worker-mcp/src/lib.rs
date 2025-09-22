//! MCP Server for Cloudflare Workers using the SDK
//!
//! This example demonstrates using the pmcp SDK directly in a Cloudflare Worker
//! now that the SDK supports WASM compilation.

use pmcp::server::wasm_core::WasmServerCore;
use pmcp::server::wasi_adapter::WasiHttpAdapter;
use serde_json::{json, Value};
use std::sync::Arc;
use worker::{console_log, Context, Env, Method, Request, Response, Result};

/// Handle HTTP requests
#[worker::event(fetch)]
pub async fn main(mut req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    console_log!("Received request: {} {}", req.method(), req.path());

    // Only handle POST requests to /mcp
    if req.method() != Method::Post || req.path() != "/mcp" {
        return Response::error("Not Found", 404);
    }

    // Get request body
    let body = req.text().await?;
    console_log!("Request body: {}", body);

    // Create server with tools
    let mut server = WasmServerCore::new(
        "cloudflare-mcp-server".to_string(),
        "1.0.0".to_string(),
    );

    // Add weather tool
    server.add_tool(
        "weather".to_string(),
        "Get weather information for a location".to_string(),
        |args: Value| {
            let location = args
                .get("location")
                .and_then(|v| v.as_str())
                .unwrap_or("San Francisco");

            Ok(json!({
                "location": location,
                "temperature": "72°F",
                "conditions": "Sunny",
                "humidity": "45%",
                "forecast": "Clear skies expected throughout the day"
            }))
        },
    );

    // Add calculator tool
    server.add_tool(
        "calculator".to_string(),
        "Perform arithmetic calculations".to_string(),
        |args: Value| {
            let operation = args
                .get("operation")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    pmcp::Error::protocol(
                        pmcp::ErrorCode::INVALID_PARAMS,
                        "operation is required",
                    )
                })?;

            let a = args.get("a").and_then(|v| v.as_f64()).ok_or_else(|| {
                pmcp::Error::protocol(pmcp::ErrorCode::INVALID_PARAMS, "parameter 'a' is required")
            })?;

            let b = args.get("b").and_then(|v| v.as_f64()).ok_or_else(|| {
                pmcp::Error::protocol(pmcp::ErrorCode::INVALID_PARAMS, "parameter 'b' is required")
            })?;

            let result = match operation {
                "add" => a + b,
                "subtract" => a - b,
                "multiply" => a * b,
                "divide" => {
                    if b == 0.0 {
                        return Err(pmcp::Error::protocol(
                            pmcp::ErrorCode::INVALID_PARAMS,
                            "Division by zero",
                        ));
                    }
                    a / b
                }
                _ => {
                    return Err(pmcp::Error::protocol(
                        pmcp::ErrorCode::INVALID_PARAMS,
                        &format!("Unknown operation: {}", operation),
                    ))
                }
            };

            Ok(json!({
                "operation": operation,
                "a": a,
                "b": b,
                "result": result
            }))
        },
    );

    // Add system info tool
    server.add_tool(
        "system_info".to_string(),
        "Get system information".to_string(),
        |_args: Value| {
            Ok(json!({
                "runtime": "Cloudflare Workers",
                "architecture": "wasm32",
                "sdk_version": env!("CARGO_PKG_VERSION"),
                "message": "MCP SDK running successfully in WASM!"
            }))
        },
    );

    // Process request using WASI adapter
    let handler = Arc::new(server);
    let adapter = WasiHttpAdapter::new();
    
    match adapter.handle_request(handler, body).await {
        Ok(response_body) => {
            console_log!("Response: {}", response_body);
            Response::ok(response_body)
                .and_then(|mut r| {
                    let _ = r.headers_mut().set("Content-Type", "application/json");
                    Ok(r)
                })
        }
        Err(e) => {
            console_log!("Error processing request: {:?}", e);
            Response::error(&format!("Error: {}", e), 500)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_server_creation() {
        let server = WasmServerCore::new(
            "test-server".to_string(),
            "1.0.0".to_string(),
        );
        // Basic sanity check
        assert!(true);
    }
}