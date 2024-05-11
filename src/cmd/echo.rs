use crate::cmd::{extract_args, validate_command, CommandError, CommandExecutor};
use crate::resp::{BulkString, RespArray, RespFrame};
use crate::Backend;

#[derive(Debug)]
pub struct Echo {
    value: String,
}

impl CommandExecutor for Echo {
    fn execute(self, _backend: &Backend) -> RespFrame {
        BulkString::new(self.value).into()
    }
}

impl TryFrom<RespArray> for Echo {
    type Error = CommandError;
    fn try_from(value: RespArray) -> Result<Self, Self::Error> {
        validate_command(&value, &["echo"], 1)?;
        let mut args = extract_args(value, 1)?.into_iter();
        match args.next() {
            Some(RespFrame::BulkString(key)) => Ok(Echo {
                value: String::from_utf8(key.0)?,
            }),
            _ => Err(CommandError::InvalidArgument("Invalid key!".to_string())),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    #[test]
    fn test_echo_command() -> Result<()> {
        let command = Echo::try_from(RespArray::new([
            BulkString::new("echo").into(),
            BulkString::new("hello").into(),
        ]))?;
        assert_eq!(command.value, "hello");
        Ok(())
    }
}
