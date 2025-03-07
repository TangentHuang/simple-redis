use crate::cmd::sadd::SAdd;
use crate::cmd::{
    CommandError, Echo, Get, HGet, HGetAll, HMGet, HSet, Set, SisMember, Unrecognized,
};
use crate::resp::{RespArray, RespFrame};
use enum_dispatch::enum_dispatch;

#[enum_dispatch(CommandExecutor)]
#[derive(Debug)]
pub enum Command {
    Get(Get),
    Set(Set),
    HGet(HGet),
    HSet(HSet),
    HGetAll(HGetAll),
    //ECHO
    Echo(Echo),
    // HMGET
    HMGet(HMGet),
    // SADD
    SAdd(SAdd),
    // SISMEMBER
    SisMember(SisMember),
    // unrecognized command
    Unrecognized(Unrecognized),
}

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
            Some(RespFrame::BulkString(ref cmd)) => {
                match cmd.as_ref().to_ascii_lowercase().as_slice() {
                    b"get" => Ok(Get::try_from(v)?.into()),
                    b"set" => Ok(Set::try_from(v)?.into()),
                    b"hget" => Ok(HGet::try_from(v)?.into()),
                    b"hset" => Ok(HSet::try_from(v)?.into()),
                    b"hgetall" => Ok(HGetAll::try_from(v)?.into()),
                    b"echo" => Ok(Echo::try_from(v)?.into()),
                    b"hmget" => Ok(HMGet::try_from(v)?.into()),
                    b"sadd" => Ok(SAdd::try_from(v)?.into()),
                    b"sismember" => Ok(SisMember::try_from(v)?.into()),
                    _ => Ok(Unrecognized.into()),
                }
            }
            _ => Err(CommandError::InvalidCommand(
                "Command must have a BulkString as the first argument".to_string(),
            )),
        }
    }
}
