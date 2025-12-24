// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::Address;
use common_contracts::contracts::keep::Keep;
use stylus_sdk::prelude::*;

#[storage]
#[entrypoint]
pub struct Scribe;

#[public]
impl Scribe {
    pub fn accept_appointment(&mut self, scribe: Address) -> Result<(), Vec<u8>> {
        let mut storage = Keep::storage();
        if !storage.scribe.get().is_zero() {
            Err(b"Scribe already appointed")?;
        }
        storage.scribe.set(scribe);
        Ok(())
    }

    pub fn verify_signature(&mut self, data: Vec<u8>) -> Result<bool, Vec<u8>> {
        let keep = Keep::storage();
        if keep.scribe.get().is_zero() {
            Err(b"Scribe not appointed")?;
        }
        // TODO: Implement actual signature verification
        let _ = data;
        Ok(true)
    }
}
