use bytes::Bytes;
use crate::{binary_uvarint, Value, ValueExt};

#[derive(Debug, Clone)]
pub struct EncodedValue {
    pub(crate) data: Bytes,
    pub(crate) expires_sz: u8,
}

impl EncodedValue {
    pub fn decode(src: &[u8]) -> Value {
        let meta = src[0];
        let user_meta = src[1];
        let (expires_at, sz) = binary_uvarint(&src[2..]);
        let value = src[2 + sz..].to_vec().into();

        Value {
            meta,
            user_meta,
            expires_at,
            version: 0,
            value,
        }
    }

    pub fn decode_bytes(src: Bytes) -> Value {
        let meta = src[0];
        let user_meta = src[1];
        let (expires_at, sz) = binary_uvarint(&src[2..]);
        let value = src.slice(2 + sz..);

        Value {
            meta,
            user_meta,
            expires_at,
            version: 0,
            value,
        }
    }
}

impl ValueExt for EncodedValue {
    #[inline]
    fn parse_value(&self) -> &[u8] {
        &self.data[(2 + self.expires_sz) as usize..]
    }

    #[inline]
    fn parse_value_to_bytes(&self) -> Bytes {
        self.data.slice((2 + self.expires_sz) as usize ..)
    }

    #[inline]
    fn get_meta(&self) -> u8 {
        self.data[0]
    }

    #[inline]
    fn get_user_meta(&self) -> u8 {
        self.data[1]
    }

    #[inline]
    fn get_expires_at(&self) -> u64 {
        let (expires_at, _) = binary_uvarint(&self.data[2..]);
        expires_at
    }
}