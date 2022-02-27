use crate::{KeyExt, KeyRef, ValueRef};
use enum_dispatch::enum_dispatch;

/// Helper struct for iterator
#[derive(Copy, Clone, Debug)]
pub enum SeekFrom {
    /// The start position
    Origin,
    /// The current position
    Current,
}

/// Custom iterator
#[enum_dispatch]
pub trait Iterator {
    /// advance to next
    fn next(&mut self);

    /// reset to 0
    fn rewind(&mut self);

    /// seek will reset iterator and seek to >= key.
    fn seek<K: KeyExt>(&mut self, key: K);

    /// Returns the entry of current position
    fn entry(&self) -> Option<(KeyRef, ValueRef)>;

    /// Returns the key of current position
    fn key(&self) -> Option<KeyRef>;

    /// Returns the value of current position
    fn val(&self) -> Option<ValueRef>;

    /// Returns if the current position has a valid value.
    fn valid(&self) -> bool;

    /// Size hint for this iterator
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }

    /// How many items in this iterator
    #[inline]
    fn count(&self) -> usize
        where
            Self: Sized,
    {
        match self.size_hint().1 {
            None => usize::MAX,
            Some(v) => v,
        }
    }
}