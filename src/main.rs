use tokio::net::TcpListener;
use tcp_async_tokio::run::run_server;

// tokio's runtime for async handling
#[tokio::main]
async fn main() -> anyhow::Result<()> { // update fn signature to return anyhow
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    // # src/run.rs
    run_server(listener).await
}
