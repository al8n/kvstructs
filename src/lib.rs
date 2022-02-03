//! General basic key-value structs for Key-Value based storages.
//!
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![deny(missing_docs)]

macro_rules! has_prefix {
    ($trait:tt::$fn:tt) => {
        /// Returns whether the slice self begins with prefix.
        #[inline]
        fn has_prefix(&self, prefix: impl $trait) -> bool {
            let src = $trait::$fn(self);
            let prefix = $trait::$fn(&prefix);
            let pl = prefix.len();
            if src.len() < pl {
                return false;
            }

            src[0..pl].eq(prefix)
        }
    };
}

macro_rules! has_suffix {
    ($trait:tt::$fn:tt) => {
        /// Returns whether the slice self ends with suffix.
        #[inline]
        fn has_suffix(&self, suffix: impl $trait) -> bool {
            let src = $trait::$fn(self);
            let suffix = $trait::$fn(&suffix);
            let pl = suffix.len() - 1;
            if src.len() <= pl {
                return false;
            }

            src[pl..].eq(suffix)
        }
    };
}

macro_rules! longest_prefix {
    ($trait:tt::$fn:tt, $ty: ty) => {
        /// Finds the longest shared prefix
        #[inline]
        fn longest_prefix(&self, other: impl $trait) -> &[$ty] {
            let k1 = $trait::$fn(self);
            let k2 = $trait::$fn(&other);
            let max = k1.len().min(k2.len());

            let mut n = max - 1;
            for i in 0..max {
                if k1[i].ne(&k2[i]) {
                    n = i;
                    break;
                }
            }
            &k1[..n]
        }
    };
}

macro_rules! longest_suffix {
    ($trait:tt::$fn:tt, $ty: ty) => {
        /// Finds the longest shared suffix
        #[inline]
        fn longest_suffix(&self, other: impl $trait) -> &[$ty] {
            let k1 = $trait::$fn(self);
            let k1_len = k1.len();
            let k2 = $trait::$fn(&other);
            let k2_len = k2.len();
            return if k1_len < k2_len {
                let max = k1_len;
                let mut n = max;
                for i in 0..max {
                    if k1[k1_len - i - 1].ne(&k2[k2_len - i - 1]) {
                        n = i;
                        break;
                    }
                }
                &k1[max - n..]
            } else {
                let max = k2_len;
                let mut n = max;
                for i in 0..max {
                    if k1[k1_len - i - 1].ne(&k2[k2_len - i - 1]) {
                        n = i;
                        break;
                    }
                }
                &k1[k1_len - k2_len + max - n..]
            };
        }
    }
}

macro_rules! longest_prefix_lossy {
    ($trait:tt::$fn:tt, $ty: ty, $ty_literal: literal) => {
        #[doc = concat!("Finds the longest shared prefix, return a Cow<'_, [", $ty_literal, "]>.")]
        #[inline]
        fn longest_prefix_lossy(&self, other: impl $trait) -> Cow<'_, [$ty]> {
            Cow::Borrowed(self.longest_prefix(other))
        }
    };
}

macro_rules! longest_suffix_lossy {
    ($trait:tt::$fn:tt, $ty: ty, $ty_literal: literal) => {
        #[doc = concat!("Finds the longest shared suffix, return a Cow<'_, [", $ty_literal, "]>.")]
        #[inline]
        fn longest_suffix_lossy(&self, other: impl $trait) -> Cow<'_, [$ty]> {
            Cow::Borrowed(self.longest_suffix(other))
        }
    };
}

macro_rules! impl_psfix_suites {
    ($trait:tt::$fn:tt, $ty: ty, $ty_literal: literal) => {
        has_prefix!($trait::$fn);

        has_suffix!($trait::$fn);

        longest_prefix!($trait::$fn, $ty);

        longest_suffix!($trait::$fn, $ty);

        longest_prefix_lossy!($trait::$fn, $ty, $ty_literal);

        longest_suffix_lossy!($trait::$fn, $ty, $ty_literal);
    };
}

extern crate alloc;

mod key;

use alloc::vec::Vec;
pub use key::*;

mod key_mut;
pub use key_mut::*;

mod utils;
mod value;
mod value_enc;
pub use value_enc::EncodedValue;
mod value_mut;
pub use value_mut::*;

pub use value::*;

#[inline]
fn u64_big_endian(b: &[u8]) -> u64 {
    (b[7] as u64)
        | ((b[6] as u64) << 8)
        | (b[5] as u64) << 16
        | (b[4] as u64) << 24
        | (b[3] as u64) << 32
        | (b[2] as u64) << 40
        | (b[1] as u64) << 48
        | (b[0] as u64) << 56
}

const MAX_VARINT_LEN64: usize = 10;

/// binary_uvarint decodes a uint64 from buf and returns that value and the
/// number of bytes read (> 0). If an error occurred, the value is 0
/// and the number of bytes n is <= 0 meaning:
///
/// n == 0: buf too small
/// n  < 0: value larger than 64 bits (overflow)
/// 	    and !n is the number of bytes read
///
#[inline]
fn binary_uvarint(buf: &[u8]) -> (u64, usize) {
    let mut x = 0;
    let mut s = 0usize;
    for (idx, b) in buf.iter().enumerate() {
        let b = *b;
        if b < 0x80 {
            if idx >= MAX_VARINT_LEN64 || idx == MAX_VARINT_LEN64 - 1 && b > 1 {
                return (0, !(idx + 1)); //overflow
            }
            return (x | (b as u64) << s, idx + 1);
        }
        x |= ((b & 0x7f) as u64) << s;
        s += 7;
    }
    (0, 0)
}

#[inline]
fn put_binary_uvarint_to_vec(vec: &mut Vec<u8>, mut x: u64) {
    while x >= 0x80 {
        vec.push((x as u8) | 0x80);
        x >>= 7;
    }
    vec.push(x as u8)
}

#[inline]
fn binary_uvarint_allocate(mut x: u64) -> Vec<u8> {
    let mut vec = Vec::with_capacity(MAX_VARINT_LEN64);
    while x >= 0x80 {
        vec.push((x as u8) | 0x80);
        x >>= 7;
    }
    vec.push(x as u8);
    vec
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        eprintln!("here");
    }
}
