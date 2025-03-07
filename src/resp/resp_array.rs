use crate::resp::{
    calc_total_length, extract_fixed_data, parse_length, RespDecode, RespEncode, RespError,
    RespFrame, BUF_CAP, CRLF_LEN,
};
use bytes::{Buf, BytesMut};
use std::ops::Deref;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct RespArray(pub(crate) Vec<RespFrame>);

impl RespArray {
    pub fn new(v: impl Into<Vec<RespFrame>>) -> Self {
        RespArray(v.into())
    }
}

// - array: "*<number-of-elements>\r\n<element-1>...<element-n>"
impl RespEncode for RespArray {
    fn encode(self) -> Vec<u8> {
        if self.0.is_empty() {
            return "*-1\r\n".into();
        }
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("*{}\r\n", self.len()).into_bytes());
        for frame in self.0 {
            buf.extend_from_slice(&frame.encode());
        }
        buf
    }
}

impl RespDecode for RespArray {
    const PREFIX: &'static str = "*";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        if len == -1 {
            extract_fixed_data(buf, "*-1\r\n", "NullArray")?;
            return Ok(RespArray::new(vec![]));
        }
        if len < -1 {
            return Err(RespError::InvalidFrameLength(len));
        }
        let len = len as usize;
        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;
        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }

        buf.advance(end + CRLF_LEN);

        let mut frames = Vec::with_capacity(len);
        for _ in 0..len {
            frames.push(RespFrame::decode(buf)?);
        }
        Ok(RespArray::new(frames))
    }

    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        if len == -1 {
            return Ok(4);
        }
        if len < -1 {
            return Err(RespError::InvalidFrameLength(len));
        }
        let len = len as usize;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

impl Deref for RespArray {
    type Target = Vec<RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::resp::{BulkString, SimpleError, SimpleString};
    #[test]
    fn test_array_encode() {
        let frame: RespFrame = RespArray(vec![
            SimpleString::new("OK").into(),
            SimpleError::new("Error message").into(),
            123i64.into(),
            BulkString::new(b"Hello, World!").into(),
        ])
        .into();
        assert_eq!(
            frame.encode(),
            b"*4\r\n+OK\r\n-Error message\r\n:123\r\n$13\r\nHello, World!\r\n"
        );
    }
    #[test]
    fn test_array_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n$5\r\nhello\r\n");

        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::new([b"set".into(), b"hello".into()]));

        buf.extend_from_slice(b"*2\r\n$3\r\nset\r\n");
        let ret = RespArray::decode(&mut buf);
        assert_eq!(ret.unwrap_err(), RespError::NotComplete);

        buf.extend_from_slice(b"$5\r\nhello\r\n");
        let frame = RespArray::decode(&mut buf)?;
        assert_eq!(frame, RespArray::new([b"set".into(), b"hello".into()]));

        Ok(())
    }

    #[test]
    fn test_null_array_encode() {
        let frame = RespArray::new(vec![]);
        assert_eq!(frame.encode(), b"*-1\r\n");
    }

    #[test]
    fn test_null_array_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"*-1\r\n");

        let frame = RespArray::decode(&mut buf)?;
        let expected = RespArray::new(vec![]);
        assert_eq!(frame, expected);

        Ok(())
    }
}
