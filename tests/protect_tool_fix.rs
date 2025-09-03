// Test to verify protect_tool fix
#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use pmcp::{RequestHandlerExtra, Result, Server, ToolHandler};
    use serde_json::{json, Value};

    struct TestTool;

    #[async_trait]
    impl ToolHandler for TestTool {
        async fn handle(&self, _args: Value, _extra: RequestHandlerExtra) -> Result<Value> {
            Ok(json!({
                "type": "text",
                "text": "Test response"
            }))
        }
    }

    #[test]
    fn test_protect_tool_without_custom_authorizer() {
        // Test 1: protect_tool should work when no custom authorizer is set
        let result = Server::builder()
            .name("test-server")
            .version("1.0.0")
            .tool("test_tool", TestTool)
            .protect_tool("test_tool", vec!["admin".to_string()])
            .protect_tool("another_tool", vec!["user".to_string()])
            .build();

        assert!(
            result.is_ok(),
            "Should build successfully with protect_tool"
        );
    }

    #[test]
    fn test_protect_tool_with_existing_authorizer_fails() {
        struct CustomAuthorizer;

        #[async_trait]
        impl pmcp::server::auth::ToolAuthorizer for CustomAuthorizer {
            async fn can_access_tool(
                &self,
                _auth: &pmcp::server::auth::AuthContext,
                _tool_name: &str,
            ) -> Result<bool> {
                Ok(true)
            }

            async fn required_scopes_for_tool(&self, _tool_name: &str) -> Result<Vec<String>> {
                Ok(vec![])
            }
        }

        // Test: Using protect_tool after setting custom authorizer should fail at build
        let result = Server::builder()
            .name("test-server")
            .version("1.0.0")
            .tool("test_tool", TestTool)
            .tool_authorizer(CustomAuthorizer)
            .protect_tool("test_tool", vec!["admin".to_string()])
            .build();

        assert!(
            result.is_err(),
            "Should fail when using both protect_tool and custom authorizer"
        );
        if let Err(e) = result {
            assert!(
                e.to_string().contains("Cannot use protect_tool"),
                "Error message should mention conflict"
            );
        }
    }
}
