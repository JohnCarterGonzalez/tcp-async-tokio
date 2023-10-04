use anyhow::Result;
use tokio::net::TcpListener;

use crate::stream::stream_handler;

pub async fn run_server(listener: TcpListener) -> Result<()> {
    loop {
        let (stream, _addr) = listener.accept().await?;

        tokio::spawn(async move {
            if let Err(e) = stream_handler(stream).await {
                eprintln!("{}", e);
            }
        });
    }
}
