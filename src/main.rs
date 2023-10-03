use tokio::{
    io::{
        AsyncBufReadExt,
        AsyncWriteExt,
        BufReader
    },
    net::{
        TcpStream,
        TcpListener
    }}; //tokio magic

async fn stream_handler(mut stream: TcpStream) -> anyhow::Result<()> {
    let (read, mut write) = stream.split();

    let mut buf_read = BufReader::new(read);
    let mut buf = String::new();

    while buf_read.read_line(&mut buf).await? > 0 {
        println!("{}", buf);

        if buf.to_ascii_uppercase().starts_with("PING") {
            write.write_all(b"+PONG\r\n").await?;
        }
         buf.clear();
    }

    Ok(())
}

// tokio's runtime for async handling
#[tokio::main]
async fn main() -> anyhow::Result<()> { // update fn signature to return anyhow
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    // handle incoming connections
    loop {
        let (stream, _addr) = listener.accept().await?;
        stream_handler(stream).await?;
    }
}
