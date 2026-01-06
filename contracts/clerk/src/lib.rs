// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U128};
use common::log_msg;
use common_contracts::
    contracts::{clerk::ClerkStorage, keep_calls::KeepCalls}
;
use stylus_sdk::{ArbResult, abi::Bytes, prelude::*};

#[storage]
#[entrypoint]
pub struct Clerk;

#[public]
impl Clerk {
    // TODO: Add UUPS (ERC-1967) so that Clerk can be behind the Gate

    pub fn initialize(&mut self, owner: Address, abacus: Address) -> Result<(), Vec<u8>> {
        let mut storage = ClerkStorage::storage();
        storage.initialize(owner, abacus)?;
        Ok(())
    }

    pub fn store(&mut self, id: U128, data: Bytes) -> Result<(), Vec<u8>> {
        let mut storage = ClerkStorage::storage();
        storage.only_owner(self.attendee())?;

        log_msg!("Storing vector");
        storage.store_bytes(id, data);

        Ok(())
    }

    pub fn load(&self, id: U128) -> Result<Bytes, Vec<u8>> {
        let storage = ClerkStorage::storage();
        storage.only_owner(self.attendee())?;

        let Some(vector) = storage.fetch_bytes(id) else {
            return Err(b"Not found".to_vec());
        };

        Ok(Bytes::from(vector))
    }

    #[payable]
    #[fallback]
    fn fallback(&mut self, calldata: &[u8]) -> ArbResult {
        let storage = ClerkStorage::storage();
        let abacus = storage.get_abacus_address();
        if abacus.is_zero() {
            Err(b"No requests implementation")?;
        }

        log_msg!("Delegating function to {}", abacus);
        unsafe { Ok(self.vm().delegate_call(&self, abacus, calldata)?) }
    }
}
