use pmcp::{Server, ServerCapabilities, ToolHandler};
use async_trait::async_trait;
use serde_json::{json, Value};

struct TestTool;

#[async_trait]
impl ToolHandler for TestTool {
    async fn handle(&self, _args: Value, _extra: pmcp::RequestHandlerExtra) -> pmcp::Result<Value> {
        Ok(json!({"message": "test"}))
    }
}

#[tokio::main]
async fn main() -> pmcp::Result<()> {
    let server = Server::builder()
        .name("test-server")
        .version("1.0.0")
        .capabilities(ServerCapabilities::tools_only())
        .tool("test", TestTool)
        .build()?;

    server.run_stdio().await?;
    Ok(())
}