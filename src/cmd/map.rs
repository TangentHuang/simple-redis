use crate::backend::Backend;
use crate::cmd::{
    extract_args, validate_command, CmdGet, CmdSet, CommandError, CommandExecutor, RESP_OK,
};
use crate::resp::{RespArray, RespFrame, RespNull};

impl CommandExecutor for CmdGet {
    fn execute(self, backend: &Backend) -> RespFrame {
        backend.get(&self.key).unwrap_or(RespFrame::Null(RespNull))
    }
}

impl CommandExecutor for CmdSet {
    fn execute(self, backend: &Backend) -> RespFrame {
        backend.set(self.key, self.value);
        RESP_OK.clone()
    }
}

impl TryFrom<RespArray> for CmdGet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["get"], 1)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(key)) => Ok(CmdGet {
                key: String::from_utf8(key.0)?,
            }),
            _ => Err(CommandError::InvalidArgument("Invalid key!".to_string())),
        }
    }
}

impl TryFrom<RespArray> for CmdSet {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["set"], 2)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(key)), Some(value)) => Ok(CmdSet {
                key: String::from_utf8(key.0)?,
                value,
            }),
            _ => Err(CommandError::InvalidArgument(
                "Invalid key or value!".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::resp::RespDecode;
    use anyhow::Result;
    use bytes::BytesMut;

    #[test]
    fn test_get_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nget\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let result: CmdGet = frame.try_into()?;
        assert_eq!(result.key, "hello");

        Ok(())
    }

    #[test]
    fn test_set_from_resp_array() -> Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*3\r\n$3\r\nset\r\n$5\r\nhello\r\n$5\r\nworld\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let result: CmdSet = frame.try_into()?;
        assert_eq!(result.key, "hello");
        assert_eq!(result.value, RespFrame::BulkString(b"world".into()));

        Ok(())
    }

    #[test]
    fn test_set_get_command() -> Result<()> {
        let backend = Backend::new();
        let cmd = CmdSet {
            key: "hello".to_string(),
            value: RespFrame::BulkString(b"world".into()),
        };
        let result = cmd.execute(&backend);
        assert_eq!(result, RESP_OK.clone());

        let cmd = CmdGet {
            key: "hello".to_string(),
        };
        let result = cmd.execute(&backend);
        assert_eq!(result, RespFrame::BulkString(b"world".into()));

        Ok(())
    }
}
