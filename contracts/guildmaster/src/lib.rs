// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U128};
use alloy_sol_types::SolCall;
use deli::contracts::{
    interfaces::{clerk::IClerk, granary::IGranary},
    keep::Keep,
};
use stylus_sdk::prelude::*;

#[storage]
#[entrypoint]
pub struct Guildmaster;

impl Guildmaster {
    fn _attendee(&self) -> Address {
        self.vm().msg_sender()
    }

    fn _send_to_granary(
        &mut self,
        gate_to_granary: Address,
        call: impl SolCall,
    ) -> Result<Vec<u8>, Vec<u8>> {
        let calldata = call.abi_encode();
        let result = self.vm().call(&self, gate_to_granary, &calldata)?;
        Ok(result)
    }

    fn _send_to_clerk(&mut self, code: Vec<u8>, num_registry: u128) -> Result<(), Vec<u8>> {
        let storage = Keep::storage();
        let gate_to_granary = storage.granary.get_granary_address();

        let call = IClerk::executeCall { code, num_registry };
        self.vm().call(&self, gate_to_granary, &call.abi_encode())?;
        Ok(())
    }
}

#[public]
impl Guildmaster {
    /// Submit new Index
    ///
    /// Deploys Vault contract in inactive state. Needs to be voted to activate.
    ///
    pub fn submit_index(
        &mut self,
        index: U128,
        asset_names: Vec<u8>,
        asset_weights: Vec<u8>,
        info: Vec<u8>,
    ) -> Result<(), Vec<u8>> {
        let mut storage = Keep::storage();
        let mut vault = storage.vaults.setter(index);

        if !vault.assets.get().is_zero() {
            return Err(b"Vault already exists".into());
        }

        let gate_to_granary = storage.granary.get_granary_address();
        let asset_names_id = storage.granary.next_vector();
        let asset_weights_id = storage.granary.next_vector();

        let set_asset_names = IGranary::storeCall {
            id: asset_names_id.to(),
            data: asset_names,
        };

        let set_asset_weights = IGranary::storeCall {
            id: asset_weights_id.to(),
            data: asset_weights,
        };

        self._send_to_granary(gate_to_granary, set_asset_names)?;
        self._send_to_granary(gate_to_granary, set_asset_weights)?;

        vault.assets.set(asset_names_id);
        vault.weights.set(asset_weights_id);

        let _ = info;
        vault
            .gate_to_vault
            .set(todo!("Deploy Gate and Vault contracts..."));

        Ok(())
    }

    /// Submit a vote for an Index
    ///
    /// Once enough votes, Vault contract is activated.
    ///
    pub fn submit_vote(&mut self, index: U128, vote: Vec<u8>) -> Result<(), Vec<u8>> {
        let mut storage = Keep::storage();
        let vault = storage.vaults.setter(index);

        if vault.assets.get().is_zero() {
            Err(b"Vault not found")?;
        }

        let _ = vault.gate_to_vault;
        let _ = vote;
        todo!("Send vote to Vault contract");

        Ok(())
    }
}

#[cfg(test)]
mod test {}
