#[cfg(test)]
mod tests {
    use super::super::simple_tool::{SimpleTool, SyncTool};
    use crate::server::cancellation::RequestHandlerExtra;
    use crate::server::ToolHandler;
    use serde_json::{json, Value};

    #[tokio::test]
    async fn test_simple_tool_with_schema() {
        let tool = SimpleTool::new(
            "test_tool",
            Box::new(|args: Value, _extra: RequestHandlerExtra| {
                Box::pin(async move {
                    let x = args.get("x").and_then(|v| v.as_i64()).unwrap_or(0);
                    Ok(json!({ "result": x * 2 }))
                })
                    as std::pin::Pin<
                        Box<dyn std::future::Future<Output = crate::Result<Value>> + Send>,
                    >
            }),
        )
        .with_description("Test tool that doubles input")
        .with_schema(json!({
            "type": "object",
            "properties": {
                "x": {
                    "type": "integer",
                    "description": "Input value to double"
                }
            },
            "required": ["x"]
        }));

        // Test metadata
        let metadata = tool.metadata().unwrap();
        assert_eq!(metadata.name, "test_tool");
        assert_eq!(
            metadata.description,
            Some("Test tool that doubles input".to_string())
        );
        assert_eq!(
            metadata.input_schema["properties"]["x"]["description"],
            "Input value to double"
        );

        // Test execution
        let extra = RequestHandlerExtra {
            cancellation_token: tokio_util::sync::CancellationToken::new(),
            request_id: "test-req".to_string(),
            session_id: None,
            auth_info: None,
            auth_context: None,
        };
        let result = tool.handle(json!({ "x": 5 }), extra).await.unwrap();
        assert_eq!(result["result"], 10);
    }

    #[tokio::test]
    async fn test_sync_tool_with_schema() {
        let tool = SyncTool::new("sync_tool", |args| {
            let msg = args
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("default");
            Ok(json!({ "echo": msg }))
        })
        .with_description("Sync echo tool")
        .with_schema(json!({
            "type": "object",
            "properties": {
                "message": {
                    "type": "string",
                    "description": "Message to echo"
                }
            }
        }));

        // Test metadata
        let metadata = tool.metadata().unwrap();
        assert_eq!(metadata.name, "sync_tool");
        assert_eq!(metadata.description, Some("Sync echo tool".to_string()));
        assert_eq!(
            metadata.input_schema["properties"]["message"]["type"],
            "string"
        );

        // Test execution
        let extra = RequestHandlerExtra {
            cancellation_token: tokio_util::sync::CancellationToken::new(),
            request_id: "test-req".to_string(),
            session_id: None,
            auth_info: None,
            auth_context: None,
        };
        let result = tool
            .handle(json!({ "message": "hello" }), extra)
            .await
            .unwrap();
        assert_eq!(result["echo"], "hello");
    }

    #[test]
    fn test_default_schema() {
        let tool = SyncTool::new("default_tool", |_args| Ok(json!({})));

        let metadata = tool.metadata().unwrap();
        assert_eq!(metadata.name, "default_tool");
        assert_eq!(metadata.description, None);
        // Check default schema allows any properties
        assert_eq!(metadata.input_schema["type"], "object");
        assert_eq!(metadata.input_schema["additionalProperties"], true);
    }

    #[tokio::test]
    async fn test_tool_error_handling() {
        let tool = SimpleTool::new(
            "error_tool",
            Box::new(|_args: Value, _extra: RequestHandlerExtra| {
                Box::pin(async move { Err(crate::Error::validation("Test error")) })
                    as std::pin::Pin<
                        Box<dyn std::future::Future<Output = crate::Result<Value>> + Send>,
                    >
            }),
        );

        let extra = RequestHandlerExtra {
            cancellation_token: tokio_util::sync::CancellationToken::new(),
            request_id: "test-error".to_string(),
            session_id: None,
            auth_info: None,
            auth_context: None,
        };
        let result = tool.handle(json!({}), extra).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Test error"));
    }

    #[test]
    fn test_tool_info_structure() {
        let tool = SyncTool::new("info_test", |_| Ok(json!({})))
            .with_description("Tool for testing ToolInfo structure")
            .with_schema(json!({
                "type": "object",
                "properties": {
                    "param1": {
                        "type": "string",
                        "enum": ["option1", "option2"]
                    },
                    "param2": {
                        "type": "number",
                        "minimum": 0,
                        "maximum": 100
                    }
                },
                "required": ["param1"]
            }));

        let info = tool.metadata().unwrap();

        // Verify the ToolInfo structure matches expected format
        assert_eq!(info.name, "info_test");
        assert!(info.description.is_some());

        // Check schema details
        let schema = &info.input_schema;
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["param1"]["enum"].is_array());
        assert_eq!(schema["properties"]["param2"]["minimum"], 0);
        assert_eq!(schema["required"][0], "param1");
    }
}
