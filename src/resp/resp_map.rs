use crate::resp::{
    calc_total_length, parse_length, RespDecode, RespEncode, RespError, RespFrame, SimpleString,
    BUF_CAP, CRLF_LEN,
};
use bytes::{Buf, BytesMut};
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub struct RespMap(pub(crate) BTreeMap<String, RespFrame>);

impl RespMap {
    pub fn new() -> Self {
        RespMap(BTreeMap::new())
    }
}

// - map: "%<number-of-entries>\r\n<key-1><value-1>...<key-n><value-n>"
impl RespEncode for RespMap {
    fn encode(self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(BUF_CAP);
        buf.extend_from_slice(&format!("%{}\r\n", self.len()).into_bytes());
        for (key, value) in self.0 {
            buf.extend_from_slice(&SimpleString::new(key).encode());
            buf.extend_from_slice(&value.encode());
        }
        buf
    }
}

impl RespDecode for RespMap {
    const PREFIX: &'static str = "%";
    fn decode(buf: &mut BytesMut) -> Result<Self, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let len = len as usize;
        let total_len = calc_total_length(buf, end, len, Self::PREFIX)?;
        if buf.len() < total_len {
            return Err(RespError::NotComplete);
        }
        buf.advance(len + CRLF_LEN);

        let mut map = RespMap::new();
        for _ in 0..len {
            let key = SimpleString::decode(buf)?;
            let value = RespFrame::decode(buf)?;
            map.insert(key.0, value);
        }
        Ok(map)
    }
    fn expect_length(buf: &[u8]) -> Result<usize, RespError> {
        let (end, len) = parse_length(buf, Self::PREFIX)?;
        let len = len as usize;
        calc_total_length(buf, end, len, Self::PREFIX)
    }
}

impl Default for RespMap {
    fn default() -> Self {
        Self::new()
    }
}
impl DerefMut for RespMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Deref for RespMap {
    type Target = BTreeMap<String, RespFrame>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::resp::BulkString;
    #[test]
    fn test_map_encode() {
        let mut map = RespMap::new();
        map.insert(
            "hello".to_string(),
            BulkString::new("world".to_string()).into(),
        );
        map.insert("foo".to_string(), (-123456.789).into());

        let frame: RespFrame = map.into();
        assert_eq!(
            &frame.encode(),
            b"%2\r\n+foo\r\n,-123456.789\r\n+hello\r\n$5\r\nworld\r\n"
        );
    }

    #[test]
    fn test_map_decode() -> anyhow::Result<()> {
        let mut buf = BytesMut::new();
        buf.extend_from_slice(b"%2\r\n+hello\r\n$5\r\nworld\r\n+foo\r\n$3\r\nbar\r\n");

        let frame = RespMap::decode(&mut buf)?;
        let mut map = RespMap::new();
        map.insert(
            "hello".to_string(),
            BulkString::new(b"world".to_vec()).into(),
        );
        map.insert("foo".to_string(), BulkString::new(b"bar".to_vec()).into());
        assert_eq!(frame, map);

        Ok(())
    }
}
