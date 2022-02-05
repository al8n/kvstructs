use core::ops::Deref;
use core::slice;
use bytes::Bytes;
use crate::{binary_uvarint, ValueExt, ValueRef};

/// RawValuePointer contains a raw pointer of the data of [`Value`]
/// This struct is unsafe, because it does not promise the raw pointer always valid.
///
/// [`Value`]: struct.Value.html
#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) struct RawValuePointer {
    ptr: *const u8,
    l: u32,
    expires_at: u64,
}

impl RawValuePointer {
    /// Returns a RawValuePointer.
    ///
    /// # Safety
    /// The inner raw pointer must be valid.
    #[inline(always)]
    pub unsafe fn new(ptr: *const u8, len: u32, expires_at: u64) -> Self {
        Self { ptr, l: len, expires_at, }
    }
}

impl Deref for RawValuePointer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.ptr, self.l as usize) }
    }
}

impl ValueExt for RawValuePointer {
    #[inline]
    fn parse_value(&self) -> &[u8] {
        &self[2..]
    }

    #[inline]
    fn parse_value_to_bytes(&self) -> Bytes {
        Bytes::copy_from_slice(&self[2..])
    }

    #[inline]
    fn get_meta(&self) -> u8 {
        self[0]
    }

    #[inline]
    fn get_user_meta(&self) -> u8 {
        self[1]
    }

    #[inline]
    fn get_expires_at(&self) -> u64 {
        self.expires_at
    }
}