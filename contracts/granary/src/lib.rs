// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U128};
use alloy_sol_types::SolCall;
use deli::{
    contracts::{
        calls::InnerCall, granary::GranaryStorage, interfaces::clerk::IClerk, keep_calls::KeepCalls,
    },
    log_msg,
};
use stylus_sdk::prelude::*;

#[storage]
#[entrypoint]
pub struct Granary;

#[public]
impl Granary {
    // TODO: Add UUPS (ERC-1967) so that Granary can be behind the Gate

    pub fn initialize(&mut self, owner: Address, clerk: Address) -> Result<(), Vec<u8>> {
        let mut storage = GranaryStorage::storage();
        storage.initialize(owner, clerk)?;
        Ok(())
    }

    pub fn store(&mut self, id: U128, data: Vec<u8>) -> Result<(), Vec<u8>> {
        let mut storage = GranaryStorage::storage();
        storage.only_owner(self.attendee())?;

        log_msg!("Storing vector");
        storage.store_bytes(id, data);

        Ok(())
    }

    pub fn load(&self, id: U128) -> Result<Vec<u8>, Vec<u8>> {
        let storage = GranaryStorage::storage();
        storage.only_owner(self.attendee())?;

        let Some(vector) = storage.fetch_bytes(id) else {
            return Err(b"Not found".to_vec());
        };

        Ok(vector)
    }

    pub fn execute(&mut self, code: Vec<u8>, num_registry: u128) -> Result<Vec<u8>, Vec<u8>> {
        let storage = GranaryStorage::storage();
        storage.only_owner(self.attendee())?;

        log_msg!("Executing code");
        let result = self.inner_call(
            storage.get_clerk_address(),
            IClerk::executeCall { code, num_registry },
        )?;

        Ok(result)
    }
}
