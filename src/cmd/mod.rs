mod command;
mod echo;
mod get;
mod hget;
mod hgetall;
mod hmget;
mod hset;
mod set;

use crate::backend;
use crate::backend::Backend;
pub use crate::cmd::command::Command;
pub use crate::cmd::{
    echo::Echo, get::Get, hget::HGet, hgetall::HGetAll, hmget::HMGet, hset::HSet, set::Set,
};
use crate::resp::{RespArray, RespError, RespFrame, SimpleString};
use enum_dispatch::enum_dispatch;
use lazy_static::lazy_static;
use thiserror::Error;
//also can use onecell
lazy_static! {
    static ref RESP_OK: RespFrame = SimpleString::new("OK").into();
}

#[derive(Error, Debug)]
pub enum CommandError {
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("{0}")]
    RespError(#[from] RespError),
    #[error("Utf8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
}
#[enum_dispatch]
pub trait CommandExecutor {
    fn execute(self, backend: &backend::Backend) -> RespFrame;
}

#[derive(Debug)]
pub struct Unrecognized;

impl CommandExecutor for Unrecognized {
    fn execute(self, _: &Backend) -> RespFrame {
        RESP_OK.clone()
    }
}

fn validate_command(
    value: &RespArray,
    names: &[&'static str],
    n_args: usize,
) -> Result<(), CommandError> {
    if value.len() < n_args + names.len() {
        return Err(CommandError::InvalidArgument(format!(
            "{} command must have exactly {} argument",
            names.join(" "),
            n_args
        )));
    }

    for (i, name) in names.iter().enumerate() {
        let name = name.to_lowercase();
        match value[i] {
            RespFrame::BulkString(ref cmd) => {
                if cmd.0.to_ascii_lowercase() != name.as_bytes() {
                    return Err(CommandError::InvalidCommand(format!(
                        "Invalid command: expected {}, got {}",
                        name,
                        String::from_utf8_lossy(cmd.as_ref())
                    )));
                }
            }
            _ => {
                return Err(CommandError::InvalidCommand(
                    "Command must have a BulkString as the first argument".to_string(),
                ))
            }
        }
    }
    Ok(())
}

fn extract_args(value: RespArray, start: usize) -> Result<Vec<RespFrame>, CommandError> {
    Ok(value.0.into_iter().skip(start).collect::<Vec<RespFrame>>())
}

#[cfg(test)]
mod test {
    use crate::cmd::command::Command;
    use crate::cmd::CommandExecutor;
    use crate::resp::{RespArray, RespDecode, RespFrame, RespNull};
    use crate::Backend;
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_command() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let cmd: Command = frame.try_into()?;

        let backend = Backend::new();

        let ret = cmd.execute(&backend);
        assert_eq!(ret, RespFrame::Null(RespNull));

        Ok(())
    }
}
