use pmcp::{Server, ServerCapabilities};

#[tokio::main]
async fn main() -> pmcp::Result<()> {
    // Minimal server without any auth or tools
    let server = Server::builder()
        .name("minimal-test")
        .version("1.0.0")
        .capabilities(ServerCapabilities::default())
        .build()?;

    server.run_stdio().await?;
    Ok(())
}