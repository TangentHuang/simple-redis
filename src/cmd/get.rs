use crate::cmd::{extract_args, validate_command, CommandError, CommandExecutor};
use crate::resp::{RespArray, RespFrame, RespNull};
use crate::Backend;

#[derive(Debug)]
pub struct Get {
    key: String,
}

impl CommandExecutor for Get {
    fn execute(self, backend: &Backend) -> RespFrame {
        backend.get(&self.key).unwrap_or(RespFrame::Null(RespNull))
    }
}

impl TryFrom<RespArray> for Get {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["get"], 1)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(key)) => Ok(Get {
                key: String::from_utf8(key.0)?,
            }),
            _ => Err(CommandError::InvalidArgument("Invalid key!".to_string())),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cmd::{Set, RESP_OK};
    use crate::resp::RespDecode;
    use bytes::BytesMut;
    #[test]
    fn test_get_from_resp_array() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let result: Get = frame.try_into()?;
        assert_eq!(result.key, "hello");

        Ok(())
    }

    #[test]
    fn test_set_get_command() -> anyhow::Result<()> {
        let backend = Backend::new();
        let cmd = Set {
            key: "hello".to_string(),
            value: RespFrame::BulkString(b"world".into()),
        };
        let result = cmd.execute(&backend);
        assert_eq!(result, RESP_OK.clone());

        let cmd = Get {
            key: "hello".to_string(),
        };
        let result = cmd.execute(&backend);
        assert_eq!(result, RespFrame::BulkString(b"world".into()));

        Ok(())
    }
}
