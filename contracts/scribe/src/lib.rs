// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::Address;
use deli::contracts::keep::Keep;
use stylus_sdk::prelude::*;

#[storage]
#[entrypoint]
pub struct Scribe;

#[public]
impl Scribe {
    pub fn accept_appointment(&mut self, castle: Address) -> Result<(), Vec<u8>> {
        let mut storage = Keep::storage();
        if !storage.castle.get() != castle {
            Err(b"Wrong Castle")?;
        }
        if !storage.scribe.get().is_zero() {
            Err(b"Scribe already appointed")?;
        }
        storage.scribe.set(self.vm().contract_address());
        Ok(())
    }

    pub fn verify_signature(&mut self, data: Vec<u8>) -> Result<bool, Vec<u8>> {
        let keep = Keep::storage();
        if keep.scribe.get() != self.vm().contract_address() {
            Err(b"Scribe not appointed")?;
        }
        // TODO: Implement actual signature verification
        let _ = data;
        Ok(true)
    }
}
