// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::U128;
use common_contracts::contracts::{keep::Keep, keep_calls::KeepCalls};
use stylus_sdk::{abi::Bytes, prelude::*};

#[storage]
#[entrypoint]
pub struct Guildmaster;

#[public]
impl Guildmaster {
    /// Submit new Index
    ///
    /// Deploys Vault contract in inactive state. Needs to be voted to activate.
    ///
    pub fn submit_index(
        &mut self,
        index: U128,
        asset_names: Bytes,
        asset_weights: Bytes,
        info: Bytes,
    ) -> Result<(), Vec<u8>> {
        let mut storage = Keep::storage();
        let mut vault = storage.vaults.setter(index);

        if !vault.assets.get().is_zero() {
            return Err(b"Vault already exists".into());
        }

        let gate_to_clerk_chamber = storage.clerk_chamber.get_gate_address();
        let asset_names_id = storage.clerk_chamber.next_vector();
        let asset_weights_id = storage.clerk_chamber.next_vector();

        self.submit_vector_bytes(gate_to_clerk_chamber, asset_names_id.to(), asset_names.0)?;
        self.submit_vector_bytes(gate_to_clerk_chamber, asset_weights_id.to(), asset_weights.0)?;

        vault.assets.set(asset_names_id);
        vault.weights.set(asset_weights_id);

        let worksman = storage.worksman.get();
        let gate_to_vault = self.build_vault(worksman, index.to(), info.0)?;

        vault.gate_to_vault.set(gate_to_vault);

        Ok(())
    }

    /// Submit a vote for an Index
    ///
    /// Once enough votes, Vault contract is activated.
    ///
    pub fn submit_vote(&mut self, index: U128, vote: Bytes) -> Result<(), Vec<u8>> {
        let mut storage = Keep::storage();
        let vault = storage.vaults.setter(index);

        if vault.assets.get().is_zero() {
            Err(b"Vault not found")?;
        }

        let scribe = storage.scribe.get();
        let verfication_result = self.verify_signature(scribe, vote.0)?;

        if !verfication_result {
            Err(b"Couldn't verify vote")?;
        }

        //TODO: Send vote to Vault contract to activate

        Ok(())
    }
}

#[cfg(test)]
mod test {}
