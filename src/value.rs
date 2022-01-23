use alloc::borrow::Cow;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use core::mem;
use crate::{binary_uvarint, binary_uvarint_allocate, put_binary_uvarint_to_vec};
use crate::value_enc::EncodedValue;

const VALUE_META_SIZE: usize = mem::size_of::<u8>() * 2 + mem::size_of::<u64>();
const META_OFFSET: usize = 0;
const USER_META_OFFSET: usize = 1;
const EXPIRATION_OFFSET: usize = 2;
const VERSION_OFFSET: usize = 10;
const VALUE_OFFSET: usize = 18;

/// Value represents the value info that can be associated with a key, but also the internal
/// Meta field.
///
/// # Design for Value
///
/// **Note:**
/// 1. `version` field will not be encoded, it is a helper field.
/// 2. `expiration` field will be encoded as uvarient, which means after encoded, the size of
/// this field is less or equal to 8 bytes.
///
/// ```text
/// +----------+-----------------+--------------------+--------------------+--------------------+
/// |   meta   |    user meta    |     expiration     |      version       |        value       |
/// +----------+-----------------+--------------------+--------------------+--------------------+
/// |  1 byte  |      1 byte     |      8 bytes       |      8 bytes       |       n bytes      |
/// +----------+-----------------+--------------------+--------------------+--------------------+
/// ```
#[derive(Default, Debug, Eq, PartialEq, Clone)]
#[repr(C)]
pub struct Value {
    meta: u8,
    user_meta: u8,
    expires_at: u64,
    version: u64, // This field is not serialized. Only for internal usage.
    value: Bytes,
}

impl Into<Bytes> for Value {
    fn into(self) -> Bytes {
        let mut b = BytesMut::with_capacity(VALUE_META_SIZE + self.value.len());
        b.put_u8(self.meta);
        b.put_u8(self.user_meta);
        b.put_u64(self.expires_at);
        b.extend(self.value);
        b.freeze()
    }
}

impl Value {
    pub fn new() -> Self {
        Self {
            meta: 0,
            user_meta: 0,
            expires_at: 0,
            version: 0,
            value: Bytes::new(),
        }
    }

    pub fn decode_from_bytes(buf: Bytes) -> Self {
        let meta = buf[0];
        let user_meta = buf[1];
        let (expires_at, sz) = binary_uvarint(&buf[2..]);
        let value = buf.slice(2 + sz..);

        Self {
            meta,
            user_meta,
            expires_at,
            version: 0,
            value,
        }
    }

    /// decode uses the length of the slice to infer the length of the Value field.
    pub fn decode(buf: &[u8]) -> Self {
        let meta = buf[0];
        let user_meta = buf[1];
        let (expires_at, sz) = binary_uvarint(&buf[2..]);
        let value = buf[2 + sz..].to_vec().into();

        Self {
            meta,
            user_meta,
            expires_at,
            version: 0,
            value,
        }
    }


}

impl ValueExt for Value {
    #[inline]
    fn parse_value(&self) -> &[u8] {
        self.value.as_ref()
    }

    #[inline]
    fn parse_value_to_bytes(&self) -> Bytes {
        self.value.clone()
    }

    #[inline]
    fn get_meta(&self) -> u8 {
        self.meta
    }

    #[inline]
    fn get_user_meta(&self) -> u8 {
        self.user_meta
    }

    #[inline]
    fn get_expires_at(&self) -> u64 {
        self.expires_at
    }

    #[inline]
    fn get_version(&self) -> u64 {
        self.version
    }
}

fn size_variant(mut x: u64) -> usize {
    let mut n = 0;
    loop {
        n += 1;
        x >>= 7;
        if x == 0 {
            break;
        }
    }
    n
}

macro_rules! impl_from_for_value {
    ($($ty: ty), +$(,)?) => {
        $(
        impl From<$ty> for Value {
            fn from(val: $ty) -> Self {
                Self {
                    meta: 0,
                    user_meta: 0,
                    expires_at: 0,
                    value: Bytes::from(val),
                    version: 0,
                }
            }
        }
        )*
    };
}

impl_from_for_value! {
    String,
    &'static str,
    &'static [u8],
    Vec<u8>,
    Box<[u8]>,
    Bytes,
    BytesMut,
}

pub trait ValueExt {

    fn parse_value(&self) -> &[u8];

    fn parse_value_to_bytes(&self) -> Bytes;

    fn get_meta(&self) -> u8;

    fn get_user_meta(&self) -> u8;

    fn get_expires_at(&self) -> u64;

    fn get_version(&self) -> u64;

    /// Returns a [`ValueRef`]
    ///
    /// [`ValueRef`]: struct.ValueRef.html
    #[inline]
    fn as_value_ref(&self) -> ValueRef {
        ValueRef {
            meta: self.get_meta(),
            user_meta: self.get_user_meta(),
            expires_at: self.get_expires_at(),
            version: self.get_version(),
            value: self.parse_value(),
        }
    }

    /// Returns the size of the Value when encoded
    #[inline]
    fn encoded_size(&self) -> u32 {
        let sz = self.parse_value().len() + 2; // meta, user meta.
        let enc = size_variant(self.get_expires_at());
        (sz + enc) as u32
    }

    /// Encode to a mutable slice. This function will copy the value.
    /// Use [`to_encoded`], if you want a shallow copy when encoded.
    ///
    /// # Panics
    /// This function panics if the remaining capacity of slice is less than encoded size.
    ///
    /// [`to_encoded`]: #method.to_encoded
    fn encode(&self, mut buf: &mut [u8]) {
        buf.put_u8(self.get_meta());
        buf.put_u8(self.get_user_meta());
        buf.put_slice(binary_uvarint_allocate(self.get_expires_at()).as_slice());
        buf.put_slice(self.parse_value());
    }

    /// Encode to [`EncodedValue`].
    ///
    /// This function may be optimized by the underlying type to avoid actual copies.
    /// For example, [`Value`] implementation will do a shallow copy (ref-count increment)
    ///
    /// [`EncodedValue`]: struct.EncodedValue.html
    /// [`Value`]: struct.Value.html
    #[inline]
    fn to_encoded(&self) -> EncodedValue {
        let mut data = Vec::with_capacity(VERSION_OFFSET);
        data.push(self.get_meta());
        data.push(self.get_user_meta());
        put_binary_uvarint_to_vec(data.as_mut(), self.get_expires_at());

        let meta = Bytes::from(data);
        let val = self.parse_value_to_bytes();
        let enc_len = meta.len() + val.len();
        EncodedValue::from(meta.chain(val).copy_to_bytes(enc_len))
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ValueRef<'a> {
    meta: u8,
    user_meta: u8,
    expires_at: u64,
    version: u64,
    value: &'a [u8],
}

impl<'a> ValueRef<'a> {
    #[inline]
    pub fn to_string(&self) -> String {
        String::from_utf8_lossy(self.value).to_string()
    }

    #[inline]
    pub fn to_lossy_string(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self.value)
    }

    #[inline]
    pub fn to_value(&self) -> Value {
        Value {
            meta: self.meta,
            user_meta: self.user_meta,
            expires_at: self.expires_at,
            version: self.version,
            value: Bytes::copy_from_slice(self.value),
        }
    }

    #[inline]
    pub fn get_meta(&self) -> u8 {
        self.meta
    }

    #[inline]
    pub fn get_user_meta(&self) -> u8 {
        self.user_meta
    }

    #[inline]
    pub fn get_expires_at(&self) -> u64 {
        self.expires_at
    }

    #[inline]
    pub fn get_val(&self) -> &[u8] {
        self.value
    }

    #[inline]
    pub fn get_version(&self) -> u64 { self.version }
}
