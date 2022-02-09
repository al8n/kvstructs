use core::ops::Deref;
use crate::{Value, ValueRef};

/// RawValuePointer contains a raw pointer of the data of [`Value`]
/// This struct is unsafe, because it does not promise the raw pointer always valid.
///
/// [`Value`]: struct.Value.html
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct RawValuePointer {
    ptr: *const Value,
    l: u32,
    expires_at: u64,
}

impl RawValuePointer {
    /// Returns a RawValuePointer.
    ///
    /// # Safety
    /// The inner raw pointer must be valid.
    #[inline(always)]
    pub const unsafe fn new(ptr: *const Value, len: u32, expires_at: u64) -> Self {
        Self { ptr, l: len, expires_at, }
    }

    #[inline(always)]
    pub unsafe fn as_value_ref(&self) -> ValueRef {
        ValueRef {
            val: &*self.ptr
        }
    }
}

impl Deref for RawValuePointer {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}
