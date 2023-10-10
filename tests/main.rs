#[cfg(test)]
mod tests {
    use redis_starter_rust::run::run_server;
    use std::net::{IpAddr, SocketAddr};
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};
    use tokio::time::timeout;
    async fn setup_server(port: u16) -> anyhow::Result<SocketAddr> {
        let listener =
            TcpListener::bind(SocketAddr::new(IpAddr::from([127, 0, 0, 1]), port)).await?;

        let addr = listener.local_addr().expect("could not get local addr");
        tokio::spawn(run_server(listener));
        Ok(addr)
    }

    async fn setup_random_server() -> anyhow::Result<SocketAddr> {
        setup_server(0).await
    }

    async fn send_and_expect_response(
        stream: &mut TcpStream,
        input: &[u8],
        expected: &[u8],
        timeout_sec: Option<u64>,
    ) -> anyhow::Result<()> {
        // Write data to the stream
        stream.write_all(input).await?;
        // Prepare a buffer to read into
        let mut buffer = vec![0u8; expected.len()];
        // If a timeout is specified, enforce it
        if let Some(t) = timeout_sec {
            timeout(Duration::from_secs(t), stream.read_exact(&mut buffer)).await??;
        } else {
            stream.read_exact(&mut buffer).await?;
        }

        // Check if the received data matches the expected data
        assert_eq!(buffer[..], expected[..]);
        Ok(())
    }

    #[tokio::test]
    async fn test_bind_to_a_port() -> anyhow::Result<()> {
        let port = 6379;
        let addr = setup_server(port).await?;
        let _stream = TcpStream::connect(addr).await?;
        Ok(())
    }

    #[tokio::test]
    async fn respond_to_a_ping() -> anyhow::Result<()> {
        let addr = setup_random_server().await?;
        let mut stream = TcpStream::connect(addr).await?;
        send_and_expect_response(&mut stream, b"+PING\r\n", b"+PONG\r\n", Some(1)).await
    }

    #[tokio::test]
    async fn respond_to_multiple_pings() -> anyhow::Result<()> {
        let addr = setup_random_server().await?;
        let mut stream = TcpStream::connect(addr).await?;
        send_and_expect_response(
            &mut stream,
            b"+PING\r\n+PING\r\n",
            b"+PONG\r\n+PONG\r\n",
            Some(1),
        )
        .await
    }

    #[tokio::test]
    async fn handle_concurrent_clients() -> anyhow::Result<()> {
        let addr = setup_random_server().await?;
        let mut stream1 = TcpStream::connect(addr).await?;
        let mut stream2 = TcpStream::connect(addr).await?;
        let mut stream3 = TcpStream::connect(addr).await?;
        send_and_expect_response(&mut stream1, b"+PING\r\n", b"+PONG\r\n", Some(1)).await?;
        send_and_expect_response(&mut stream2, b"+PING\r\n", b"+PONG\r\n", Some(1)).await?;
        send_and_expect_response(&mut stream3, b"+PING\r\n", b"+PONG\r\n", Some(1)).await?;

        Ok(())
    }

    #[tokio::test]
    async fn implement_the_echo_command() -> anyhow::Result<()> {
        let addr = setup_random_server().await?;
        let mut stream = TcpStream::connect(addr).await?;
        send_and_expect_response(
            &mut stream,
            b"*2\r\n$4\r\nECHO\r\n$3\r\nhey\r\n",
            b"$3\r\nhey\r\n",
            Some(1),
        )
        .await
    }

    #[tokio::test]
    async fn implement_the_set_get_commands() -> anyhow::Result<()> {
        let addr = setup_random_server().await?;
        let mut stream = TcpStream::connect(addr).await?;
        // SET hey ho and GET hey
        send_and_expect_response(
            &mut stream,
            b"*3\r\n$3\r\nSET\r\n$3\r\nhey\r\n$2\r\nho\r\n",
            b"+OK\r\n",
            Some(1),
        )
        .await?;
        send_and_expect_response(
            &mut stream,
            b"*2\r\n$3\r\nGET\r\n$3\r\nhey\r\n",
            b"$2\r\nho\r\n",
            Some(1),
        )
        .await?;
        // GET non_existing
        send_and_expect_response(
            &mut stream,
            b"*2\r\n$3\r\nGET\r\n$12\r\nnon_existing\r\n",
            b"_\r\n",
            Some(1),
        )
        .await?;
        // overwrite hey and GET hey
        send_and_expect_response(
            &mut stream,
            b"*3\r\n$3\r\nSET\r\n$3\r\nhey\r\n$5\r\nhello\r\n",
            b"+OK\r\n",
            Some(1),
        )
        .await?;
        send_and_expect_response(
            &mut stream,
            b"*2\r\n$3\r\nGET\r\n$3\r\nhey\r\n",
            b"$5\r\nhello\r\n",
            Some(1),
        )
        .await?;
        Ok(())
    }

    #[tokio::test]
    async fn expiry() -> anyhow::Result<()> {
        let addr = setup_random_server().await?;

        let mut stream = TcpStream::connect(addr).await?;

        // SET hey ho and GET hey
        send_and_expect_response(
            &mut stream,
            b"*5\r\n$3\r\nSET\r\n$3\r\nhey\r\n$2\r\nho\r\n$2\r\nPX\r\n$3\r\n100\r\n",
            b"+OK\r\n",
            Some(1),
        )
        .await?;
        send_and_expect_response(
            &mut stream,
            b"*2\r\n$3\r\nGET\r\n$3\r\nhey\r\n",
            b"$2\r\nho\r\n",
            Some(1),
        )
        .await?;
        tokio::time::sleep(Duration::from_millis(150)).await;
        send_and_expect_response(
            &mut stream,
            b"*2\r\n$3\r\nGET\r\n$3\r\nhey\r\n",
            b"$-1\r\n",
            Some(1),
        )
        .await?;
        Ok(())
    }
}
