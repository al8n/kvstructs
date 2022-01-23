//! General basic key-value structs for Key-Value based storages.
//!
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![deny(missing_docs)]

extern crate alloc;

mod key;

use alloc::vec::Vec;
pub use key::*;

mod key_mut;
pub use key_mut::*;

mod utils;
mod value;
mod value_enc;

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
mod tests {}
