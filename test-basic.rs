use pmcp::{Server, ServerCapabilities};

#[tokio::main]
async fn main() -> pmcp::Result<()> {
    let server = Server::builder()
        .name("test")
        .version("1.0.0")
        .capabilities(ServerCapabilities::default())
        .build()?;
    
    server.run_stdio().await?;
    Ok(())
}