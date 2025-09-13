//! WASI-specific server implementation for MCP.

use crate::shared::cancellation::{CancellationToken, RequestHandlerExtra};
use crate::server::traits::RequestHandler;
use crate::types::{RequestId, JSONRPCResponse};
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

/// A lightweight runner for executing a `RequestHandler` in a WASI environment.
///
/// This runner is stateless and handles a single request at a time.
/// It is designed to be called by a WASI host.
#[derive(Debug)]
pub struct RequestHandlerWasiRunner<H: RequestHandler> {
    handler: Arc<H>,
}

impl<H: RequestHandler> RequestHandlerWasiRunner<H> {
    /// Create a new WASI runner with the given request handler.
    pub fn new(handler: H) -> Self {
        Self {
            handler: Arc::new(handler),
        }
    }

    /// Handle a single incoming request.
    ///
    /// This is the main entry point for the WASI runner. The host should call
    /// this function with the raw request body, and it will return the raw
    /// response body.
    pub async fn run(&self, request_body: &[u8]) -> Vec<u8> {
        match serde_json::from_slice::<Value>(request_body) {
            Ok(json_req) => {
                let response = self.handle_single_request(json_req).await;
                serde_json::to_vec(&response).unwrap_or_else(|e| {
                    serde_json::to_vec(&serde_json::json!({
                        "jsonrpc": "2.0",
                        "error": {"code": -32603, "message": format!("Internal error: {}", e)},
                        "id": null
                    }))
                    .unwrap()
                })
            }
            Err(e) => serde_json::to_vec(&serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": -32700, "message": format!("Parse error: {}", e)},
                "id": null
            }))
            .unwrap(),
        }
    }

    /// Handle a single JSON-RPC request.
    async fn handle_single_request(&self, req: Value) -> JSONRPCResponse {
        let id = req
            .get("id")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_else(|| RequestId::from(Uuid::new_v4().to_string()));

        let method = req.get("method").and_then(|v| v.as_str());
        let params = req.get("params").cloned().unwrap_or(Value::Null);

        let cancellation_token = CancellationToken::new();
        let _extra = RequestHandlerExtra::new(id.to_string(), cancellation_token);

        let result = match method {
            Some("tools/list") => {
                let list_req = serde_json::from_value(params).unwrap();
                self.handler.list_tools(list_req).await.map(|r| serde_json::to_value(r).unwrap())
            }
            Some("tools/call") => {
                let call_req = serde_json::from_value(params).unwrap();
                self.handler.call_tool(call_req).await.map(|r| serde_json::to_value(r).unwrap())
            }
            _ => Err(crate::Error::method_not_found(method.unwrap_or_default())),
        };

        match result {
            Ok(value) => JSONRPCResponse {
                jsonrpc: "2.0".to_string(),
                id,
                payload: crate::types::jsonrpc::ResponsePayload::Result(value),
            },
            Err(e) => JSONRPCResponse {
                jsonrpc: "2.0".to_string(),
                id,
                payload: crate::types::jsonrpc::ResponsePayload::Error(e.into()),
            },
        }
    }
}
