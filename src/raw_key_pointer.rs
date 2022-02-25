use core::cmp::Ordering;
use crate::{Key, KeyExt, KeyRef};
use core::ops::Deref;
use core::slice;

/// RawKeyPointer contains a raw pointer of the data slice of [`Key`]
/// This struct is unsafe, because it does not promise the raw pointer always valid.
///
/// [`Key`]: struct.Key.html
#[derive(Debug, Copy, Clone)]
pub struct RawKeyPointer {
    ptr: *const u8,
    l: u32,
}

impl From<Key> for RawKeyPointer {
    fn from(k: Key) -> Self {
        RawKeyPointer {
            ptr: k.as_slice().as_ptr(),
            l: k.as_slice().len() as u32,
        }
    }
}

impl<'a> From<KeyRef<'a>> for RawKeyPointer {
    fn from(k: KeyRef<'a>) -> Self {
        Self {
            ptr: k.as_slice().as_ptr(),
            l: k.as_slice().len() as u32,
        }
    }
}

impl RawKeyPointer {
    /// Returns a RawKeyPointer
    ///
    /// # Safety
    /// The inner raw pointer must be valid.
    #[inline(always)]
    pub const unsafe fn new(ptr: *const u8, len: u32) -> Self {
        Self { ptr, l: len }
    }

    /// Converts RawKeyPointer to KeyRef
    ///
    /// # Safety
    /// The inner raw pointer must be valid.
    #[inline(always)]
    pub unsafe fn as_key_ref(&self) -> KeyRef<'_> {
        KeyRef::from(self as &[u8])
    }
}

impl Deref for RawKeyPointer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.ptr, self.l as usize) }
    }
}

impl KeyExt for RawKeyPointer {
    #[inline]
    fn as_bytes(&self) -> &[u8] {
        self
    }
}

impl PartialEq<RawKeyPointer> for RawKeyPointer {
    fn eq(&self, other: &RawKeyPointer) -> bool {
        self.ptr.eq(&other.ptr)
    }
}

impl Eq for RawKeyPointer {}

impl PartialOrd<RawKeyPointer> for RawKeyPointer {
    fn partial_cmp(&self, other: &RawKeyPointer) -> Option<Ordering> {
        unsafe {
            self.as_key_ref().partial_cmp(&other.as_key_ref())
        }
    }
}

impl Ord for RawKeyPointer {
    fn cmp(&self, other: &Self) -> Ordering {
        unsafe {
            self.as_key_ref().cmp(&other.as_key_ref())
        }
    }
}