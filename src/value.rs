use alloc::borrow::Cow;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use core::mem;
use crate::{binary_uvarint, binary_uvarint_allocate, put_binary_uvariant_to_vec};
use crate::value_enc::EncodedValue;

const VALUE_META_SIZE: usize = mem::size_of::<u8>() * 2 + mem::size_of::<u64>();
const META_OFFSET: usize = 0;
const USER_META_OFFSET: usize = 1;
const EXPIRATION_OFFSET: usize = 2;
const VERSION_OFFSET: usize = 10;
const VALUE_OFFSET: usize = 18;

/// Value represents the value info that can be associated with a key, but also the internal
/// Meta field. The data in the Value is not mutable.
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
/// |   meta   |    user meta    |     expiration     |      version       |        data        |
/// +----------+-----------------+--------------------+--------------------+--------------------+
/// |  1 byte  |      1 byte     |      8 bytes       |      8 bytes       |       n bytes      |
/// +----------+-----------------+--------------------+--------------------+--------------------+
/// ```
#[derive(Default, Debug, Eq, PartialEq, Clone)]
#[repr(C)]
pub struct Value {
    pub(crate) meta: u8,
    pub(crate) user_meta: u8,
    pub(crate) expires_at: u64,
    pub(crate) version: u64, // This field is not serialized. Only for internal usage.
    pub(crate) value: Bytes,
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
    /// Returns an empty value
    #[inline]
    pub const fn new() -> Self {
        Self {
            meta: 0,
            user_meta: 0,
            expires_at: 0,
            version: 0,
            value: Bytes::new(),
        }
    }

    /// Returns the version for this value
    #[inline]
    pub fn get_version(&self) -> u64 {
        self.version
    }

    /// Returns a [`ValueRef`]
    ///
    /// [`ValueRef`]: struct.ValueRef.html
    #[inline]
    pub fn as_value_ref(&self) -> ValueRef {
        ValueRef {
            val: self
        }
    }

    /// Returns the number of bytes contained in the value data.
    #[inline]
    pub fn len(&self) -> usize {
        self.value.len()
    }

    /// Returns true if the value data has a length of 0.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
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

/// Extensions for `Value`
pub trait ValueExt {
    /// Returns the value data
    fn parse_value(&self) -> &[u8];

    /// Returns the value data, shallow copy
    fn parse_value_to_bytes(&self) -> Bytes;

    /// Get the meta of the value
    fn get_meta(&self) -> u8;

    /// Get the user meta
    fn get_user_meta(&self) -> u8;

    /// Returns the expiration time (unix timestamp) for this value
    fn get_expires_at(&self) -> u64;

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
        put_binary_uvariant_to_vec(data.as_mut(), self.get_expires_at());

        let expires_sz = data.len() - 2;
        let meta = Bytes::from(data);
        let val = self.parse_value_to_bytes();
        let enc_len = meta.len() + val.len();

        EncodedValue {
            data: meta.chain(val).copy_to_bytes(enc_len),
            expires_sz: expires_sz as u8
        }
    }


    /// Decodes value from slice.
    #[inline]
    fn decode(src: &[u8]) -> Value {
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

    /// Decode value from Bytes
    #[inline]
    fn decode_bytes(src: Bytes) -> Value {
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

    impl_psfix_suites!(ValueExt::parse_value, u8, "u8");
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct ValueRef<'a> {
    val: &'a Value,
}

impl<'a> ValueRef<'a> {
    /// Converts a slice of bytes to a string, including invalid characters.
    #[inline]
    pub fn to_string(&self) -> String {
        String::from_utf8_lossy(self.val.value.as_ref()).to_string()
    }

    /// Converts a slice of bytes to a string, including invalid characters.
    #[inline]
    pub fn to_lossy_string(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(self.val.value.as_ref())
    }

    /// Copy the data to a new value
    #[inline]
    pub fn to_value(&self) -> Value {
        Value {
            meta: self.val.meta,
            user_meta: self.val.user_meta,
            expires_at: self.val.expires_at,
            version: self.val.version,
            value: self.val.value.clone(),
        }
    }

    /// Get the value version
    #[inline]
    pub fn get_version(&self) -> u64 { self.val.version }
}

impl<'a> ValueExt for ValueRef<'a> {
    #[inline]
    fn parse_value(&self) -> &[u8] {
        self.val.value.as_ref()
    }

    #[inline]
    fn parse_value_to_bytes(&self) -> Bytes {
        self.val.value.clone()
    }

    #[inline]
    fn get_meta(&self) -> u8 {
        self.val.meta
    }

    #[inline]
    fn get_user_meta(&self) -> u8 {
        self.val.user_meta
    }

    #[inline]
    fn get_expires_at(&self) -> u64 {
        self.val.expires_at
    }
}
