use tokio::io::{AsyncReadExt, AsyncWriteExt}; // more better tokio!!!

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:6379").await?;

    while let Ok((mut stream, _)) = listener.accept().await {
        let mut buf = bytes::BytesMut::new();
        if stream.read(&mut buf).await.is_ok() {
            println!("{}", String::from_utf8_lossy(&buf));
            stream.write(b"+PONG\r\n").await.ok();
        }
    }
    Ok(())
}
