use pmcp::{Server, ServerCapabilities};

#[tokio::main]
async fn main() -> pmcp::Result<()> {
    eprintln!("Starting minimal server...");

    let server = Server::builder()
        .name("test")
        .version("1.0.0")
        .capabilities(ServerCapabilities::default())
        .build()?;

    eprintln!("Server built, running stdio...");
    server.run_stdio().await?;
    Ok(())
}
