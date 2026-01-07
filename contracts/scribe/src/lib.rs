// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use common_contracts::contracts::keep::Keep;
use stylus_sdk::{abi::Bytes, prelude::*};

#[storage]
#[entrypoint]
pub struct Scribe;

#[public]
impl Scribe {
    pub fn verify_signature(&mut self, data: Bytes) -> Result<bool, Vec<u8>> {
        let keep = Keep::storage();
        if keep.scribe.get().is_zero() {
            Err(b"Scribe not appointed")?;
        }
        // TODO: Implement actual signature verification
        let _ = data;
        Ok(true)
    }
}
