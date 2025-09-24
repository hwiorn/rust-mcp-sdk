//! Tests for WASM-compatible server core.

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::super::wasm_core::WasmServerCore;
    use crate::error::ErrorCode;
    use crate::server::ProtocolHandler;
    use crate::types::{ClientRequest, Request, RequestId};
    use serde_json::json;

    #[tokio::test]
    async fn test_initialize() {
        let server = WasmServerCore::new("test-server".to_string(), "1.0.0".to_string());

        let init_request = Request::Client(Box::new(
            serde_json::from_value(json!({
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "test-client",
                        "version": "1.0.0"
                    }
                }
            }))
            .unwrap(),
        ));

        let response = server
            .handle_request(RequestId::from(1i64), init_request)
            .await;

        // Check response structure
        assert_eq!(response.jsonrpc, "2.0");
        assert!(matches!(
            response.payload,
            crate::types::jsonrpc::ResponsePayload::Result(_)
        ));

        if let crate::types::jsonrpc::ResponsePayload::Result(value) = response.payload {
            assert!(value.get("protocolVersion").is_some());
            assert!(value.get("serverInfo").is_some());
            assert!(value.get("capabilities").is_some());
        }
    }

    #[tokio::test]
    async fn test_tools_list_uninitialized() {
        let server = WasmServerCore::new("test-server".to_string(), "1.0.0".to_string());

        let list_request = Request::Client(Box::new(
            serde_json::from_value(json!({
                "method": "tools/list",
                "params": {}
            }))
            .unwrap(),
        ));

        let response = server
            .handle_request(RequestId::from(1i64), list_request)
            .await;

        // Should return error when not initialized
        assert!(matches!(
            response.payload,
            crate::types::jsonrpc::ResponsePayload::Error(_)
        ));

        if let crate::types::jsonrpc::ResponsePayload::Error(error) = response.payload {
            assert_eq!(error.code, ErrorCode::INVALID_REQUEST.0);
            assert!(error.message.contains("not initialized"));
        }
    }

    #[tokio::test]
    async fn test_tools_list_after_init() {
        let mut server = WasmServerCore::new("test-server".to_string(), "1.0.0".to_string());

        // Add a tool
        server.add_tool(
            "test-tool".to_string(),
            "A test tool".to_string(),
            |_args| Ok(json!({"result": "test"})),
        );

        // Initialize first
        let init_request = Request::Client(Box::new(
            serde_json::from_value(json!({
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "test-client",
                        "version": "1.0.0"
                    }
                }
            }))
            .unwrap(),
        ));

        server
            .handle_request(RequestId::from(1i64), init_request)
            .await;

        // Now list tools
        let list_request = Request::Client(Box::new(
            serde_json::from_value(json!({
                "method": "tools/list",
                "params": {}
            }))
            .unwrap(),
        ));

        let response = server
            .handle_request(RequestId::from(2i64), list_request)
            .await;

        assert!(matches!(
            response.payload,
            crate::types::jsonrpc::ResponsePayload::Result(_)
        ));

        if let crate::types::jsonrpc::ResponsePayload::Result(value) = response.payload {
            let tools = value.get("tools").unwrap().as_array().unwrap();
            assert_eq!(tools.len(), 1);
            assert_eq!(tools[0]["name"], "test-tool");
            assert_eq!(tools[0]["description"], "A test tool");
        }
    }

    #[tokio::test]
    async fn test_tool_call_success() {
        let mut server = WasmServerCore::new("test-server".to_string(), "1.0.0".to_string());

        // Add calculator tool
        server.add_tool("add".to_string(), "Add two numbers".to_string(), |args| {
            let a = args.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let b = args.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
            Ok(json!({"result": a + b}))
        });

        // Initialize
        let init_request = Request::Client(Box::new(
            serde_json::from_value(json!({
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "test-client",
                        "version": "1.0.0"
                    }
                }
            }))
            .unwrap(),
        ));

        server
            .handle_request(RequestId::from(1i64), init_request)
            .await;

        // Call tool
        let call_request = Request::Client(Box::new(
            serde_json::from_value(json!({
                "method": "tools/call",
                "params": {
                    "name": "add",
                    "arguments": {
                        "a": 5,
                        "b": 3
                    }
                }
            }))
            .unwrap(),
        ));

        let response = server
            .handle_request(RequestId::from(2i64), call_request)
            .await;

        assert!(matches!(
            response.payload,
            crate::types::jsonrpc::ResponsePayload::Result(_)
        ));

        if let crate::types::jsonrpc::ResponsePayload::Result(value) = response.payload {
            assert_eq!(value["isError"], false);
            let content = value["content"].as_array().unwrap();
            assert_eq!(content[0]["type"], "text");
            // Tool result is stringified JSON
            let text = content[0]["text"].as_str().unwrap();
            let result: serde_json::Value = serde_json::from_str(text).unwrap();
            assert_eq!(result["result"], 8.0);
        }
    }

    #[tokio::test]
    async fn test_tool_call_unknown_tool() {
        let server = WasmServerCore::new("test-server".to_string(), "1.0.0".to_string());

        // Initialize
        let init_request = Request::Client(Box::new(
            serde_json::from_value(json!({
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "test-client",
                        "version": "1.0.0"
                    }
                }
            }))
            .unwrap(),
        ));

        server
            .handle_request(RequestId::from(1i64), init_request)
            .await;

        // Call unknown tool
        let call_request = Request::Client(Box::new(
            serde_json::from_value(json!({
                "method": "tools/call",
                "params": {
                    "name": "unknown-tool",
                    "arguments": {}
                }
            }))
            .unwrap(),
        ));

        let response = server
            .handle_request(RequestId::from(2i64), call_request)
            .await;

        assert!(matches!(
            response.payload,
            crate::types::jsonrpc::ResponsePayload::Error(_)
        ));

        if let crate::types::jsonrpc::ResponsePayload::Error(error) = response.payload {
            assert_eq!(error.code, ErrorCode::METHOD_NOT_FOUND.0);
            assert!(error.message.contains("not found"));
        }
    }

    #[tokio::test]
    async fn test_tool_call_missing_params() {
        let server = WasmServerCore::new("test-server".to_string(), "1.0.0".to_string());

        // Initialize
        let init_request = Request::Client(Box::new(
            serde_json::from_value(json!({
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "test-client",
                        "version": "1.0.0"
                    }
                }
            }))
            .unwrap(),
        ));

        server
            .handle_request(RequestId::from(1i64), init_request)
            .await;

        // Call tool without name
        let call_request = Request::Client(Box::new(
            serde_json::from_value(json!({
                "method": "tools/call",
                "params": {}
            }))
            .unwrap(),
        ));

        let response = server
            .handle_request(RequestId::from(2i64), call_request)
            .await;

        assert!(matches!(
            response.payload,
            crate::types::jsonrpc::ResponsePayload::Error(_)
        ));

        if let crate::types::jsonrpc::ResponsePayload::Error(error) = response.payload {
            assert_eq!(error.code, ErrorCode::INVALID_PARAMS.0);
        }
    }

    #[tokio::test]
    async fn test_tool_error_handling() {
        let mut server = WasmServerCore::new("test-server".to_string(), "1.0.0".to_string());

        // Add tool that returns error
        server.add_tool(
            "failing-tool".to_string(),
            "A tool that fails".to_string(),
            |_args| Err(crate::error::Error::internal("Tool execution failed")),
        );

        // Initialize
        let init_request = Request::Client(Box::new(
            serde_json::from_value(json!({
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "test-client",
                        "version": "1.0.0"
                    }
                }
            }))
            .unwrap(),
        ));

        server
            .handle_request(RequestId::from(1i64), init_request)
            .await;

        // Call failing tool
        let call_request = Request::Client(Box::new(
            serde_json::from_value(json!({
                "method": "tools/call",
                "params": {
                    "name": "failing-tool",
                    "arguments": {}
                }
            }))
            .unwrap(),
        ));

        let response = server
            .handle_request(RequestId::from(2i64), call_request)
            .await;

        // Should return success with isError: true
        assert!(matches!(
            response.payload,
            crate::types::jsonrpc::ResponsePayload::Result(_)
        ));

        if let crate::types::jsonrpc::ResponsePayload::Result(value) = response.payload {
            assert_eq!(value["isError"], true);
            let content = value["content"].as_array().unwrap();
            assert_eq!(content[0]["type"], "text");
            assert!(content[0]["text"].as_str().unwrap().contains("Error"));
        }
    }
}
