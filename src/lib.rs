//! General basic key-value structs for Key-Value based storages.
//!
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![deny(missing_docs)]

extern crate alloc;

mod key;
pub use key::*;

mod value;
mod utils;

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

#[cfg(test)]
mod tests {

    #[test]
    fn test_works() {

    }
}
