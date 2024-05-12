use crate::cmd::{extract_args, validate_command, CommandError, CommandExecutor, RESP_OK};
use crate::resp::{RespArray, RespFrame};
use crate::Backend;

#[derive(Debug)]
pub struct SAdd {
    name: String,
    values: Vec<String>,
}

impl CommandExecutor for SAdd {
    fn execute(self, backend: &Backend) -> RespFrame {
        backend.insert_set(self.name, self.values);
        RESP_OK.clone()
    }
}

impl TryFrom<RespArray> for SAdd {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["sadd"], 2)?;
        let args = extract_args(value, 1)?;
        let mut data = Vec::new();
        for arg in args {
            match arg {
                RespFrame::BulkString(s) => {
                    data.push(String::from_utf8(s.0)?);
                }
                _ => {
                    return Err(CommandError::InvalidArgument(
                        "Invalid key, field or value".to_string(),
                    ))
                }
            }
        }
        Ok(SAdd {
            name: data.remove(0),
            values: data,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::resp::RespDecode;
    use bytes::BytesMut;

    #[test]
    fn test_sadd_from_resp_array() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*4\r\n$4\r\nsadd\r\n$5\r\nmyset\r\n$5\r\nhello\r\n$5\r\nworld\r\n");

        let frame = RespArray::decode(&mut buf)?;

        let result: SAdd = frame.try_into()?;
        assert_eq!(result.name, "myset");
        assert_eq!(
            result.values,
            vec!["hello".to_string(), "world".to_string()]
        );
        Ok(())
    }
}
