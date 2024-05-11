use crate::cmd::{extract_args, validate_command, CommandError, CommandExecutor};
use crate::resp::{RespArray, RespFrame, RespNull};
use crate::Backend;

#[derive(Debug)]
pub struct HGet {
    pub(crate) key: String,
    pub(crate) field: String,
}

impl CommandExecutor for HGet {
    fn execute(self, backend: &Backend) -> RespFrame {
        backend
            .hget(&self.key, &self.field)
            .unwrap_or(RespFrame::Null(RespNull))
    }
}

impl TryFrom<RespArray> for HGet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["hget"], 2)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(RespFrame::BulkString(field))) => Ok(HGet {
                key: String::from_utf8(key.0)?,
                field: String::from_utf8(field.0)?,
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid key or field".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::resp::RespDecode;
    use bytes::BytesMut;
    #[test]
    fn test_hget_from_resp_array() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$4\r\nhget\r\n$3\r\nmap\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let result: HGet = frame.try_into()?;
        assert_eq!(result.key, "map");
        assert_eq!(result.field, "hello");

        Ok(())
    }
}
