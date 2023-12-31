use crate::redis_server::internal_state::RedisInternalState;
use anyhow::bail;
use std::collections::VecDeque;
use tokio::io::{AsyncWrite, AsyncWriteExt};
pub mod builder;

// TODO considerations
// 1. add error type for SimpleError
// 2. f64 is not Hash, Eq. How should we handle this when adding Map/Set?
// 3. How do we handle eq operation for map, set(also f64)?

// INFO: defined to represent different types of Redis Protocol Data, such as simple strings
// ints, bulk strings, arrays, and null values
#[derive(Eq, PartialEq, Debug, Clone)]
pub enum RespData {
    SimpleString(String),
    // SimpleError(String),
    Integer(i64),
    BulkString(String),
    Array(VecDeque<RespData>),
    Null,
    // Boolean(bool),
    // Double(f64),
    // BigNumber(String),
    // BulkError(Vec<u8>),
    // VerbatimString { encoding: String, value: Vec<u8> },
    // Map(HashMap<Box<RespData>, Box<RespData>>),
    // Sets(HashSet<Box<RespData>>),
    // Push(Vec<Box<RespData>>),
}

impl RespData {
    // handles different types of data and execute the corresponding command
    // if data: str -> result 'str command
    pub(crate) async fn run(self, write: &mut (impl AsyncWrite + Unpin), state: &mut RedisInternalState) -> anyhow::Result<()> {
        dbg!(&self);
        let result = match self {
            RespData::SimpleString(s) | RespData::BulkString(s) => run_simple_cmd(&s),
            RespData::Integer(_i) => unimplemented!("Integer"),
            RespData::Array(data) => run_array(data, state),
            RespData::Null => unimplemented!("Null"),
        }?;
        write.write_all(&result.to_u8_vec()).await?;
        Ok(())
    }

    pub(crate) fn to_u8_vec(&self) -> Vec<u8> {
        match self {
            RespData::SimpleString(s) => format!("+{}\r\n", s).into_bytes(),
            RespData::BulkString(s) => format!("${}\r\n{}\r\n", s.len(), s).into_bytes(),
            RespData::Integer(i) => format!(":{}\r\n", i).into_bytes(),
            RespData::Array(data) => {
                let mut result = format!("*{}\r\n", data.len()).into_bytes();
                for d in data {
                    result.extend(d.to_u8_vec());
                }
                result
            }
            // Representation of Null in the Redis Protocol -> "$-1\r\n"
            RespData::Null => "$-1\r\n".to_string().into_bytes(),
        }
    }
}

fn run_simple_cmd(cmd: &str) -> anyhow::Result<RespData> {
    match cmd.to_ascii_uppercase().as_str() {
        "PING" => Ok(RespData::SimpleString("PONG".to_string())),
        _ => bail!("run_simple_cmd for {}", cmd),
    }
}

fn run_cmd( cmd: &str, mut args: VecDeque<RespData>, state: &mut RedisInternalState, ) -> anyhow::Result<RespData> {
    match cmd.to_ascii_uppercase().as_str() {
        "PING" => run_simple_cmd(cmd),
        "ECHO" => Ok(args.pop_front().unwrap_or(RespData::Null)),
        "SET" => {
            let Some(RespData::BulkString(key) | RespData::SimpleString(key)) = args.pop_front()
            else {
                bail!("SET key is not bulkString");
            };

            let Some(RespData::BulkString(value) | RespData::SimpleString(value)) = args.pop_front()
            else {
                bail!("SET value is not bulkString");
            };

            // "PX" arg
            let is_expiry = args.pop_front().is_some();
            let mut expiry = None;

            if is_expiry {
                let arg = args.pop_front();

                if let Some(RespData::BulkString(s) | RespData::SimpleString(s)) = arg.as_ref() {
                    expiry = Some(s.parse::<i64>()?);
                }

                if let Some(RespData::Integer(i)) = arg {
                    expiry = Some(i)
                }

                if expiry.is_none() {
                   bail!("SET PX is not Integer");
                }
            }

            state.set(key, value, expiry)?;

            Ok(RespData::SimpleString("OK".to_string()))
        }
        "GET" => {
            let Some(RespData::BulkString(key) | RespData::SimpleString(key)) = args.pop_front()
            else {
                bail!("SET key is not bulkString");
            };

            let value = state.get(&key).cloned();

            match value {
                None => Ok(RespData::Null),
                Some(value) => Ok(RespData::BulkString(value)),
            }
        }
        _ => bail!("run_cmd for {}", cmd),
    }
}

fn run_array(mut data: VecDeque<RespData>, state: &mut RedisInternalState, ) -> anyhow::Result<RespData> {
    let Some(cmd) = data.pop_front() else {
        bail!("empty array");
    };
    match cmd {
        RespData::SimpleString(s) | RespData::BulkString(s) => run_cmd(&s, data, state),
        _ => bail!("run_array for {:?}", cmd),
    }
}
