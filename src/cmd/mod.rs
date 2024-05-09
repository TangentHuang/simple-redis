mod hmap;
mod map;

use crate::backend;
use crate::backend::Backend;
use crate::resp::{RespArray, RespError, RespFrame, SimpleString};
use enum_dispatch::enum_dispatch;
use lazy_static::lazy_static;
use thiserror::Error;

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

#[enum_dispatch(CommandExecutor)]
#[derive(Debug)]
pub enum Command {
    Get(CmdGet),
    Set(CmdSet),
    HGet(CmdHGet),
    HSet(CmdHSet),
    HGetAll(CmdHGetAll),
    // unrecognized command
    Unrecognized(Unrecognized),
}

#[derive(Debug)]
pub struct CmdGet {
    key: String,
}

#[derive(Debug)]
pub struct CmdSet {
    key: String,
    value: RespFrame,
}

#[derive(Debug)]
pub struct CmdHGet {
    key: String,
    field: String,
}

#[derive(Debug)]
pub struct CmdHSet {
    key: String,
    field: String,
    value: RespFrame,
}

#[derive(Debug)]
pub struct CmdHGetAll {
    key: String,
}

#[derive(Debug)]
pub struct Unrecognized;

impl TryFrom<RespFrame> for Command {
    type Error = CommandError;
    fn try_from(value: RespFrame) -> Result<Self, Self::Error> {
        match value {
            RespFrame::Array(array) => Command::try_from(array),
            _ => Err(CommandError::InvalidCommand(
                "Command must be array".to_string(),
            )),
        }
    }
}
impl TryFrom<RespArray> for Command {
    type Error = CommandError;
    fn try_from(v: RespArray) -> Result<Self, Self::Error> {
        match v.first() {
            Some(RespFrame::BulkString(ref cmd)) => match cmd.0.as_slice() {
                b"get" => Ok(CmdGet::try_from(v)?.into()),
                b"set" => Ok(CmdSet::try_from(v)?.into()),
                b"hget" => Ok(CmdHGet::try_from(v)?.into()),
                b"hset" => Ok(CmdHSet::try_from(v)?.into()),
                b"hgetall" => Ok(CmdHGetAll::try_from(v)?.into()),
                _ => Ok(Unrecognized.into()),
            },
            _ => Err(CommandError::InvalidCommand(
                "Command must have a BulkString as the first argument".to_string(),
            )),
        }
    }
}

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
    if value.len() != n_args + names.len() {
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
    #[test]
    fn test_validate_command() {}
}
