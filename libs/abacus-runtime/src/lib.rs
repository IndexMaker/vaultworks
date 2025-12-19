#![cfg_attr(all(feature = "stylus", not(feature = "stylus-test")), no_std)]

//#[macro_use]
extern crate alloc;

pub mod runtime;

#[cfg(test)]
pub mod test;