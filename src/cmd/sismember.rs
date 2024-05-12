use crate::cmd::{extract_args, validate_command, CommandError, CommandExecutor};
use crate::resp::{RespArray, RespFrame};
use crate::Backend;

#[derive(Debug)]
pub struct SisMember {
    key: String,
    value: String,
}

impl CommandExecutor for SisMember {
    fn execute(self, backend: &Backend) -> RespFrame {
        let flag = backend.sismember(&self.key, &self.value);
        match flag {
            true => RespFrame::Integer(1),
            false => RespFrame::Integer(0),
        }
    }
}
impl TryFrom<RespArray> for SisMember {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["SISMEMBER"], 2)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match (args.next(), args.next()) {
            (Some(RespFrame::BulkString(name)), Some(RespFrame::BulkString(value))) => {
                Ok(SisMember {
                    key: String::from_utf8(name.0)?,
                    value: String::from_utf8(value.0)?,
                })
            }
            _ => Err(CommandError::InvalidArgument(
                "Invalid key or value".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_sismember_command() -> anyhow::Result<()> {
        let backend = Backend::new();
        backend.insert_set("set".to_string(), vec!["a".to_string(), "b".to_string()]);
        let command = SisMember {
            key: "set".to_string(),
            value: "a".to_string(),
        };
        let ans = command.execute(&backend);
        assert_eq!(ans, RespFrame::Integer(1));
        let command = SisMember {
            key: "set".to_string(),
            value: "c".to_string(),
        };
        let ans = command.execute(&backend);
        assert_eq!(ans, RespFrame::Integer(0));
        Ok(())
    }
}
