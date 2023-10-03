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
    pub(crate) async fn run(self, write: &mut (impl AsyncWrite + Unpin)) -> anyhow::Result<()> {
        dbg!(&self);
        let result = match self {
            RespData::SimpleString(s) | RespData::BulkString(s) => run_cmd(&s, Default::default()),
            RespData::Integer(_i) => unimplemented!("Integer"),
            RespData::Array(data) => run_array(data),
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
            RespData::Null => "_\r\n".to_string().into_bytes(),
        }
    }
}

fn run_cmd(cmd: &str, mut args: VecDeque<RespData>) -> anyhow::Result<RespData> {
    match cmd.to_ascii_uppercase().as_str() {
        "PING" => Ok(RespData::SimpleString("PONG".to_string())),
        "ECHO" => Ok(args.pop_front().unwrap_or(RespData::Null)),
        _ => bail!("run_string for {}", cmd),
    }
}

fn run_array(mut data: VecDeque<RespData>) -> anyhow::Result<RespData> {
    let Some(cmd) = data.pop_front() else {
        bail!("empty array");
    };
    match cmd {
        RespData::SimpleString(s) | RespData::BulkString(s) => run_cmd(&s, data),
        _ => bail!("run_array for {:?}", cmd),
    }
}
