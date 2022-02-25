use crate::ValueRef;
use core::ops::Deref;
use core::ptr::null;
use core::slice::from_raw_parts;

/// RawValuePointer contains a raw pointer of the data of [`Value`]
/// This struct is unsafe, because it does not promise the raw pointer always valid.
///
/// [`Value`]: struct.Value.html
#[derive(Debug, Copy, Clone)]
pub struct RawValuePointer {
    pub(crate) meta: u8,
    pub(crate) user_meta: u8,
    pub(crate) version: u64, // This field is not serialized. Only for internal usage.
    pub(crate) ptr: *const u8,
    pub(crate) l: u32,
    pub(crate) expires_at: u64,
}

impl RawValuePointer {
    /// Returns a null RawValuePointer.
    ///
    /// # Safety
    /// The inner raw pointer is a null raw pointer.
    #[inline(always)]
    pub const unsafe fn new() -> Self {
        Self {
            meta: 0,
            user_meta: 0,
            version: 0,
            ptr: null(),
            l: 0,
            expires_at: 0,
        }
    }

    /// Returns a [`ValueRef`] according to the inner raw value pointer
    ///
    /// # Safety
    /// The inner raw pointer must be valid.
    #[inline(always)]
    pub unsafe fn as_value_ref(&self) -> ValueRef<'_> {
        ValueRef {
            meta: self.meta,
            user_meta: self.user_meta,
            expires_at: self.expires_at,
            version: self.version,
            val: from_raw_parts(self.ptr, self.l as usize),
        }
    }
}

impl Deref for RawValuePointer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { from_raw_parts(self.ptr, self.l as usize) }
    }
}

impl PartialEq<RawValuePointer> for RawValuePointer {
    fn eq(&self, other: &RawValuePointer) -> bool {
        self.ptr.eq(&other.ptr)
    }
}

impl Eq for RawValuePointer {}