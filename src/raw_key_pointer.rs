use core::ops::Deref;
use core::slice;
use crate::{Key, KeyExt, KeyRef};

/// RawKeyPointer contains a raw pointer of the data slice of [`Key`]
/// This struct is unsafe, because it does not promise the raw pointer always valid.
///
/// [`Key`]: struct.Key.html
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct RawKeyPointer {
    ptr: *const u8,
    l: u32,
}

impl From<Key> for RawKeyPointer {
    fn from(k: Key) -> Self {
        RawKeyPointer {
            ptr: k.as_slice().as_ptr(),
            l: k.as_slice().len() as u32
        }
    }
}

impl<'a> From<KeyRef<'a>> for RawKeyPointer  {
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
    pub unsafe fn new(ptr: *const u8, len: u32) -> Self {
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