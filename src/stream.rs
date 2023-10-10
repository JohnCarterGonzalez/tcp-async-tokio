use std::ops::ControlFlow;

use tokio::io::{ AsyncBufReadExt, BufReader };
use tokio::net::TcpStream;

use crate::resp::builder::RespBuilder;
use crate::redis_server::internal_state::RedisInternalState;

// stream_handler asynchronously handle TCP streams
// splits the read and write parts and wraps
// the read into a buffer, loops through the buffer
// if a line starts with PING, it writes back PONG
// after handling it clears the buffer
// @returns stream_handler() -> mut TcpStream -> 'Result
pub async fn stream_handler(mut stream: TcpStream) -> anyhow::Result<()> {
    let (read, mut write) = stream.split();

    let mut buf_read = BufReader::new(read);
    let mut buf = String::new();

    let mut resp_builder = RespBuilder::default();

    let mut state = RedisInternalState::default();

    while buf_read.read_line(&mut buf).await? > 0 {
        if !buf.ends_with("\r\n") {
            // if the line does not end with "\r\n\", it is not a complete req
            // continue the while loop so next line can be appended to the buf
            continue;
        }

        let current_line = buf.trim_end_matches("\r\n").to_string();
        dbg!(&current_line);

        // process the request, if complete, return a response and reset
        // resp_builder
        match resp_builder.feed_string(current_line)? {
            ControlFlow::Continue(next_resp_builder) => {
                resp_builder = next_resp_builder;
            }
            ControlFlow::Break(resp_data) => {
                dbg!(&resp_data);
                resp_data.run(&mut write, &mut state).await?;
                resp_builder = RespBuilder::default();
            }
        }

        buf.clear();
    }

    if resp_builder != RespBuilder::default() {
        dbg!("bad or incomplete request");
    }

    Ok(())
}
