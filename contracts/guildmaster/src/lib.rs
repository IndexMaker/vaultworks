// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U8, U128};
use alloy_sol_types::SolCall;
use deli::contracts::{
    interfaces::{clerk::IClerk, granary::IGranary, scribe::IScribe, worksman::IWorksman},
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

        let worksman = storage.worksman.get();
        let build_vault_calldata = IWorksman::buildVaultCall {
            index: index.to(),
            info,
        };

        let gate_to_vault_bytes = unsafe {
            self.vm()
                .delegate_call(&self, worksman, &build_vault_calldata.abi_encode())
        }?;

        vault
            .gate_to_vault
            .set(Address::from_slice(&gate_to_vault_bytes));

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

        let scribe = storage.scribe.get();
        let verify_signature_calldata = IScribe::verifySignatureCall { data: vote };

        let verification_result_bytes = unsafe {
            self.vm()
                .delegate_call(&self, scribe, &verify_signature_calldata.abi_encode())
        }?;

        let verfication_result = U8::from_be_slice(&verification_result_bytes);

        if verfication_result.is_zero() {
            Err(b"Couldn't verify vote")?;
        }

        //TODO: Send vote to Vault contract to activate

        Ok(())
    }
}

#[cfg(test)]
mod test {}
