use bytes::Bytes;
use crate::{binary_uvarint, Value, ValueExt};

/// The position store meta in a encoded value
pub const META_OFFSET: usize = 0;
/// The position store user meta in a encoded value
pub const USER_META_OFFSET: usize = 1;
/// The position store expires_at in a encoded value
pub const EXPIRATION_OFFSET: usize = 2;

/// EncodedValue contains the data need to be stored in Bytes.
///
/// **Note**: When [`Value`] is encoded to `EncodedValue`,
/// the version field will not be encoded.
/// So, when convert from `EncodedValue` to [`Value`],
/// version is always be 0.
#[derive(Debug, Clone)]
pub struct EncodedValue {
    pub(crate) data: Bytes,
    pub(crate) expires_sz: u8,
}

impl EncodedValue {
    /// Decode `EncodedValue` to Value (shallow copy).
    pub fn decode_value(&self) -> Value {
        let meta = self.data[META_OFFSET];
        let user_meta = self.data[USER_META_OFFSET];
        let (expires_at, sz) = binary_uvarint(&self.data[EXPIRATION_OFFSET..]);
        let value = self.data.slice(EXPIRATION_OFFSET + sz..);

        Value {
            meta,
            user_meta,
            expires_at,
            version: 0,
            value,
        }
    }

    /// Returns the length of encoded value
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns if the encoded value is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
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

    #[inline]
    fn to_encoded(&self) -> EncodedValue {
        self.clone()
    }
}