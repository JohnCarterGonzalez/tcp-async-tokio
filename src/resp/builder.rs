use std::collections::VecDeque;
use std::ops::ControlFlow;
use std::ops::ControlFlow::{Break, Continue};

use anyhow::bail;

use crate::resp::RespData;

#[derive(Eq, PartialEq, Default, Debug, Clone)]
pub enum RespBuilder {
    #[default]
    NotInitialized,
    ArrayBuilder {
        size: usize,
        data: VecDeque<RespData>,
        building: Box<RespBuilder>,
    },
    BulkStringBuilder {
        size: usize,
        data: String,
    },
}

impl RespBuilder {
    pub fn feed_string(self, s: String) -> anyhow::Result<ControlFlow<RespData, RespBuilder>> {
        match self {
            RespBuilder::NotInitialized => Self::inititalize(s),
            RespBuilder::ArrayBuilder {
                size,
                data,
                building,
            } => Self::feed_string_to_array_builder(size, data, *building, s),
            RespBuilder::BulkStringBuilder { size, data } => {
                Self::feed_string_to_bulk_string_builder(size, data, s)
            }
        }
    }

    fn initialize(s: String) -> anyhow::Result<ControlFlow<RespData, RespBuilder>> {
        let Some(first_byte) = s.bytes().next() else {
            bail!("empty string")
        };

        match first_byte {
            b'+' => Ok(Break(RespData::SimpleString(s.chars().skip(1).collect()))),
            b':' => {
                let num = s.chars().skip(1).collect::<String>().parse::<i64>()?;
                Ok(Break(RespData::Integer(num)))
            }
            b'*' => {
                let size = s.chars().skip(1).collect::<String>().parse::<usize>()?;
                Ok(Continue(RespBuilder::ArrayBuilder {
                    size,
                    data: VecDeque::with_capacity(size),
                    building: Box::default(),
                }))
            }
            b'$' => {
                let size = s.chars().skip(1).collect::<String>().parse::<usize>()?;
                Ok(Continue(RespBuilder::BulkStringBuilder {
                    size,
                    data: String::new(),
                }))
            }
            _ => unimplemented!("first byte {:b} is not supported", first_byte),
        }
    }
    fn feed_string_to_array_builder(
        size: usize,
        mut data: VecDeque<RespData>,
        building: RespBuilder,
        s: String,
    ) -> anyhow::Result<ControlFlow<RespData, RespBuilder>> {
        assert!(size > data.len());
        match building.feed_string(s)? {
            Continue(next) => Ok(Continue(RespBuilder::ArrayBuilder {
                size,
                data,
                building: Box::new(next),
            })),
            Break(new_data) => {
                data.push_back(new_data);
                if size == data.len() {
                    Ok(Break(RespData::Array(data)))
                } else {
                    Ok(Continue(RespBuilder::ArrayBuilder {
                        size,
                        data,
                        building: Box::default(),
                    }))
                }
            }
        }
    }

    fn feed_string_to_bulk_string_builder(
        size: usize,
        mut data: String,
        s: String,
    ) -> anyhow::Result<ControlFlow<RespData, RespBuilder>> {
        if data.len() + s.len() + 2 < size {
            data.push_str(&s);
            data.push_str("\r\n");
            return Ok(Continue(RespBuilder::BulkStringBuilder { size, data }));
        }
        if data.len() + s.len() == size {
            data.push_str(&s);
            Ok(Break(RespData::BulkString(data)))
        } else {
            bail!("bulk string size is not correct")
        }
    }
}
