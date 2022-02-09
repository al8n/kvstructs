use bytes::Bytes;
use crate::{binary_uvarint, Value, ValueExt};
/// The position store meta in a encoded value
const META_OFFSET: usize = 0;
/// The position store user meta in a encoded value
const USER_META_OFFSET: usize = 1;
/// The position store expires_at in a encoded value
const EXPIRATION_OFFSET: usize = 2;

#[derive(Debug, Clone)]
pub struct EncodedValue {
    pub(crate) data: Bytes,
    pub(crate) expires_sz: u8,
}

impl EncodedValue {
    pub fn decode(src: &[u8]) -> Value {
        let meta = src[META_OFFSET];
        let user_meta = src[USER_META_OFFSET];
        let (expires_at, sz) = binary_uvarint(&src[EXPIRATION_OFFSET..]);
        let value = src[EXPIRATION_OFFSET + sz..].to_vec().into();

        Value {
            meta,
            user_meta,
            expires_at,
            version: 0,
            value,
        }
    }

    pub fn decode_bytes(src: Bytes) -> Value {
        let meta = src[META_OFFSET];
        let user_meta = src[USER_META_OFFSET];
        let (expires_at, sz) = binary_uvarint(&src[EXPIRATION_OFFSET..]);
        let value = src.slice(EXPIRATION_OFFSET + sz..);

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
        &self.data[(EXPIRATION_OFFSET + self.expires_sz as usize)..]
    }

    #[inline]
    fn parse_value_to_bytes(&self) -> Bytes {
        self.data.slice((EXPIRATION_OFFSET + self.expires_sz as usize)..)
    }

    #[inline]
    fn get_meta(&self) -> u8 {
        self.data[META_OFFSET]
    }

    #[inline]
    fn get_user_meta(&self) -> u8 {
        self.data[USER_META_OFFSET]
    }

    #[inline]
    fn get_expires_at(&self) -> u64 {
        let (expires_at, _) = binary_uvarint(&self.data[EXPIRATION_OFFSET..]);
        expires_at
    }
}