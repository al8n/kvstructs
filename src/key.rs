use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp::Ordering;
use bytes::{Buf, Bytes, BytesMut};
use crate::u64_big_endian;

const TIMESTAMP_SIZE: usize = core::mem::size_of::<u64>();

#[derive(Debug, Clone, Hash)]
#[repr(transparent)]
pub struct Key(Bytes);

impl Default for Key {
    fn default() -> Self {
        Self::null()
    }
}

impl AsRef<[u8]> for Key {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Key {
    #[inline]
    pub const fn null() -> Self {
        Self(Bytes::new())
    }

    #[inline]
    pub fn from_utf8_slice_and_ts(data: &[u8], ts: u64) -> Self {
        let len = data.len() + 8;
        data.chain(ts.to_be_bytes().as_ref()).copy_to_bytes(len).into()
    }

    #[inline]
    pub fn from_utf8_and_ts(data: Vec<u8>, ts: u64) -> Self {
        let len = data.len() + 8;
        Bytes::from(data)
            .chain(ts.to_be_bytes().as_ref())
            .copy_to_bytes(len)
            .into()
    }

    #[inline]
    pub fn from_bytes_and_ts(data: Bytes, ts: u64) -> Self {
        let len = data.len() + 8;
        data.chain(ts.to_be_bytes().as_ref()).copy_to_bytes(len).into()
    }

    #[inline]
    pub fn copy_from_slice(data: &[u8]) -> Self {
        Bytes::copy_from_slice(data).into()
    }

    /// Generates a new key by appending timestamp to key.
    #[inline]
    pub fn with_timestamp(self, ts: u64) -> Self {
        let len = self.0.len() + 8;
        let ts = Bytes::from(Box::from((u64::MAX - ts).to_be_bytes()));
        self.0.chain(ts).copy_to_bytes(len).into()
    }


    #[inline]
    pub fn parse_key(&self) -> &[u8] {
        let sz = self.len();
        self.0[..sz - TIMESTAMP_SIZE].as_ref()
    }

    #[inline]
    pub fn parse_key_to_bytes(&self) -> Bytes {
        let sz = self.len();
        self.0.slice(..sz - TIMESTAMP_SIZE)
    }

    /// parses the timestamp from the key bytes.
    /// 
    /// # Panics
    /// If the length of key less than 8.
    #[inline]
    pub fn parse_timestamp(&self) -> u64 {
        if self.len() <= TIMESTAMP_SIZE {
            0
        } else {
            u64::MAX - u64_big_endian(&self.0[self.len() - TIMESTAMP_SIZE..])
        }
    }

    /// Returns the number of bytes contained in this Key.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the Key has a length of 0.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    
    /// Returns the underlying bytes
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_ref()
    }

    /// Checks for key equality ignoring the version timestamp.
    #[inline]
    pub fn same_key(&self, other: impl AsKeyRef) -> bool {
        same_key(self, other)
    }

    /// Checks the key without timestamp and checks the timestamp if keyNoTs
    /// is same.
    /// a<timestamp> would be sorted higher than aa<timestamp> if we use bytes.compare
    /// All keys should have timestamp.
    #[inline]
    pub fn compare_key(&self, other: impl AsKeyRef) -> Ordering {
        compare_key(self, other)
    } 
}

impl PartialEq<Self> for Key {
    fn eq(&self, other: &Self) -> bool {
        self.same_key(other)
    }
}

impl Eq for Key {}

// impl<'a> PartialEq<KeyRef<'a>> for Key {
//     fn eq(&self, other: &KeyRef<'a>) -> bool {
//         same_key(self, other)
//     }
// }
//
// impl<'a> PartialOrd<KeyRef<'a>> for Key {
//     fn partial_cmp(&self, other: &KeyRef<'a>) -> Option<Ordering> {
//         Some(compare_key(self, other))
//     }
// }

impl PartialOrd<Self> for Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Key {
    /// cmp checks the key without timestamp and checks the timestamp if keyNoTs
    /// is same.
    /// a<timestamp> would be sorted higher than aa<timestamp> if we use bytes.compare
    /// All keys should have timestamp.
    fn cmp(&self, other: &Self) -> Ordering {
        compare_key(self, other)
    }
}

/// Checks the key without timestamp and checks the timestamp if keyNoTs
/// is same.
/// a<timestamp> would be sorted higher than aa<timestamp> if we use bytes.compare
/// All keys should have timestamp.
#[inline(always)]
pub fn compare_key(a: impl AsKeyRef, b: impl AsKeyRef) -> Ordering {
    let me = a.as_key_slice();
    let other = b.as_key_slice();
    let sb = me.len().checked_sub(TIMESTAMP_SIZE).unwrap_or(0);
    let ob = other.len().checked_sub(TIMESTAMP_SIZE).unwrap_or(0);
    let (s_key_part, s_ts_part) = me.split_at(sb);
    let (o_key_part, o_ts_part) = other.split_at(ob);

    match s_key_part.cmp(o_key_part) {
        Ordering::Less => Ordering::Less,
        Ordering::Equal => s_ts_part.cmp(o_ts_part),
        Ordering::Greater => Ordering::Greater,
    }
}

#[inline(always)]
pub fn same_key(a: impl AsKeyRef, b: impl AsKeyRef) -> bool {
    let me = a.as_key_slice();
    let other = b.as_key_slice();
    let sl = me.len();
    let ol = other.len();
    if sl != ol {
        false
    } else {
        let s = &me[..sl - TIMESTAMP_SIZE];
        let o = &other[..ol - TIMESTAMP_SIZE];
        s.eq(o)
    }
}

impl<const N: usize> From<[u8; N]> for Key {
    fn from(data: [u8; N]) -> Self {
        Self(Bytes::from(data.to_vec()))
    }
}

macro_rules! impl_from_for_key {
    ($($ty: ty), +$(,)?) => {
        $(
        impl From<$ty> for Key {
            fn from(val: $ty) -> Self {
                Self(Bytes::from(val))
            }
        }
        )*
    };
}

impl_from_for_key! {
    String,
    &'static str,
    Vec<u8>,
    Box<[u8]>,
}

impl From<Bytes> for Key {
    fn from(data: Bytes) -> Self {
        Self(data)
    }
}

impl From<BytesMut> for Key {
    fn from(data: BytesMut) -> Self {
        Self(data.freeze())
    }
}

impl From<&[u8]> for Key {
    fn from(data: &[u8]) -> Self {
        Key::copy_from_slice(data)
    }
}

#[derive(Debug, Copy, Clone, Hash)]
#[repr(transparent)]
pub struct KeyRef<'a> {
    data: &'a [u8]
}

impl<'a, 'b> PartialEq<KeyRef<'b>> for KeyRef<'a> {
    fn eq(&self, other: &KeyRef<'b>) -> bool {
        same_key(self, other)
    }
}

impl<'a> Eq for KeyRef<'a> {}

impl<'a, 'b> PartialOrd<KeyRef<'b>> for KeyRef<'a> {
    fn partial_cmp(&self, other: &KeyRef<'b>) -> Option<Ordering> {
        Some(compare_key(self, other))
    }
}

impl<'a> Ord for KeyRef<'a> {
    /// Checks the key without timestamp and checks the timestamp if keyNoTs
    /// is same.
    /// a<timestamp> would be sorted higher than aa<timestamp> if we use bytes.compare
    /// All keys should have timestamp.
    fn cmp(&self, other: &Self) -> Ordering {
        compare_key(self, other)
    }
}

impl<'a> KeyRef<'a> {
    #[inline]
    pub fn to_key(&self) -> Key {
        Key::copy_from_slice(self.data)
    }

    #[inline]
    pub fn parse_key(&self) -> &[u8] {
        let sz = self.len();
        self.data[..sz - TIMESTAMP_SIZE].as_ref()
    }

    /// parses the timestamp from the key bytes.
    ///
    /// # Panics
    /// If the length of key less than 8.
    #[inline]
    pub fn parse_timestamp(&self) -> u64 {
        if self.len() <= TIMESTAMP_SIZE {
            0
        } else {
            u64::MAX - u64_big_endian(&self.data[self.len() - TIMESTAMP_SIZE..])
        }
    }

    /// Returns the number of bytes contained in this Key.
    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns true if the Key has a length of 0.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the underlying bytes
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.data.as_ref()
    }

    /// Checks for key equality ignoring the version timestamp.
    #[inline]
    pub fn same_key(&self, other: impl AsKeyRef) -> bool {
        same_key(self, other)
    }

    /// Checks the key without timestamp and checks the timestamp if keyNoTs
    /// is same.
    /// a<timestamp> would be sorted higher than aa<timestamp> if we use bytes.compare
    /// All keys should have timestamp.
    #[inline]
    pub fn compare_key(&self, other: impl AsKeyRef) -> Ordering {
        compare_key(self, other)
    }
}

impl AsKeyRef for &'_ KeyRef<'_> {
    #[inline]
    fn as_key_slice(&self) -> &[u8] {
        self.data
    }
}

impl AsKeyRef for &'_ mut KeyRef<'_> {
    #[inline]
    fn as_key_slice(&self) -> &[u8] {
        self.data
    }
}

impl AsKeyRef for KeyRef<'_> {
    #[inline]
    fn as_key_slice(&self) -> &[u8] {
        self.data
    }
}

pub trait AsKeyRef {
    #[inline]
    fn as_key_ref(&self) -> KeyRef {
        KeyRef {
            data: self.as_key_slice()
        }
    }

    fn as_key_slice(&self) -> &[u8];
}

macro_rules! impl_partial_eq_ord {
    ($($ty:ty), +$(,)?) => {
        $(
        impl PartialEq<Key> for $ty {
            fn eq(&self, other: &Key) -> bool {
                other.same_key(self)
            }
        }

        impl PartialEq<$ty> for Key {
            fn eq(&self, other: &$ty) -> bool {
                self.same_key(other)
            }
        }

        impl<'a> PartialEq<KeyRef<'a>> for $ty {
            fn eq(&self, other: &KeyRef<'a>) -> bool {
                other.same_key(self)
            }
        }

        impl<'a> PartialEq<$ty> for KeyRef<'a> {
            fn eq(&self, other: &$ty) -> bool {
                self.same_key(other)
            }
        }

        impl PartialOrd<Key> for $ty {
            fn partial_cmp(&self, other: &Key) -> Option<Ordering> {
                Some(compare_key(other, self))
            }
        }

        impl PartialOrd<$ty> for Key {
            fn partial_cmp(&self, other: &$ty) -> Option<Ordering> {
                Some(compare_key(self, other))
            }
        }

        impl<'a> PartialOrd<KeyRef<'a>> for $ty {
            fn partial_cmp(&self, other: &KeyRef<'a>) -> Option<Ordering> {
                Some(compare_key(other, self))
            }
        }

        impl<'a> PartialOrd<$ty> for KeyRef<'a> {
            fn partial_cmp(&self, other: &$ty) -> Option<Ordering> {
                Some(compare_key(self, other))
            }
        }
        )*
    };
}

macro_rules! impl_as_key_ref {
    ($($ty:tt::$conv:tt), +$(,)?) => {
        $(
        impl AsKeyRef for $ty {
            #[inline]
            fn as_key_slice(&self) -> &[u8] {
                $ty::$conv(self)
            }
        }

        impl<'a> AsKeyRef for &'a $ty {
            #[inline]
            fn as_key_slice(&self) -> &[u8] {
                $ty::$conv(self)
            }
        }

        impl<'a> AsKeyRef for &'a mut $ty {
            #[inline]
            fn as_key_slice(&self) -> &[u8] {
                $ty::$conv(self)
            }
        }
        )*
    };
}

type VecBytes = Vec<u8>;
type U8Bytes = [u8];
type BoxBytes = Box<[u8]>;

impl_partial_eq_ord! {
    Bytes,
    BytesMut,
    BoxBytes,
    U8Bytes,
    VecBytes,
    str,
    String,
}

impl_as_key_ref! {
    Bytes::as_ref,
    BytesMut::as_ref,
    BoxBytes::as_ref,
    Key::as_ref,
    U8Bytes::as_ref,
    VecBytes::as_slice,
    str::as_bytes,
    String::as_bytes,
}

impl<const N: usize> PartialEq<Key> for [u8; N] {
    fn eq(&self, other: &Key) -> bool {
        other.same_key(self)
    }
}

impl<const N: usize> PartialEq<[u8; N]> for Key {
    fn eq(&self, other: &[u8; N]) -> bool {
        self.same_key(other)
    }
}

impl<const N: usize> AsKeyRef for [u8; N] {
    #[inline]
    fn as_key_slice(&self) -> &[u8] {
        self
    }
}

impl<'a, const N: usize> AsKeyRef for &'a [u8; N] {
    #[inline]
    fn as_key_slice(&self) -> &[u8] {
        self.as_slice()
    }
}

impl<'a, const N: usize> AsKeyRef for &'a mut [u8; N] {
    #[inline]
    fn as_key_slice(&self) -> &[u8] {
        self.as_slice()
    }
}