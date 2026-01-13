// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use abacus_formulas::update_quote::update_quote;
use alloc::{string::String, vec::Vec};

use alloy_primitives::U128;
use alloy_sol_types::SolEvent;
use common::vector::Vector;
use common_contracts::{
    contracts::{
        calls::InnerCall,
        clerk::ClerkStorage,
        keep::{Keep, Vault},
        keep_calls::KeepCalls,
    },
    interfaces::{guildmaster::IGuildmaster, vault::IVault},
};
use stylus_sdk::{abi::Bytes, prelude::*};
use vector_macros::amount_vec;

#[storage]
#[entrypoint]
pub struct Guildmaster;

fn _init_vendor_quote(
    vault: &mut Vault,
    clerk_storage: &mut ClerkStorage,
    vendor_id: U128,
) -> U128 {
    let mut set_quote_id = vault.vendor_quotes.setter(vendor_id);

    let quote_id = set_quote_id.get();
    if !quote_id.is_zero() {
        return quote_id;
    }

    let quote_id = clerk_storage.next_vector();
    set_quote_id.set(quote_id);

    clerk_storage.store_vector(quote_id.to(), amount_vec![0, 0, 0]);

    if vault.vendors_bids.get(vendor_id).is_zero() && vault.vendors_asks.get(vendor_id).is_zero() {
        vault.vendors.push(vendor_id);
    }

    quote_id
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
        asset_names: Bytes,
        asset_weights: Bytes,
        name: String,
        symbol: String,
    ) -> Result<(), Vec<u8>> {
        let mut storage = Keep::storage();
        storage.check_version()?;

        let mut vault = storage.vaults.setter(index);
        if !vault.assets.get().is_zero() {
            return Err(b"Vault already exists".into());
        }

        let mut clerk_storage = ClerkStorage::storage();
        let asset_names_id = clerk_storage.next_vector();
        let asset_weights_id = clerk_storage.next_vector();

        clerk_storage.store_bytes(asset_names_id, asset_names);
        clerk_storage.store_bytes(asset_weights_id, asset_weights);

        vault.assets.set(asset_names_id);
        vault.weights.set(asset_weights_id);

        let worksman = storage.worksman.get();
        let gate_to_vault = self.build_vault(worksman, index.to(), name, symbol)?;

        vault.gate_to_vault.set(gate_to_vault);

        Ok(())
    }

    pub fn begin_edit_index(&mut self, index: U128) -> Result<(), Vec<u8>> {
        let mut storage = Keep::storage();
        let sender = self.attendee();
        storage.check_version()?;

        let vault = storage.vaults.setter(index);

        self.external_call(
            vault.gate_to_vault.get(),
            IVault::transferOwnershipCall { new_owner: sender },
        )?;

        let event = IGuildmaster::BeginEditIndex {
            index: index.to(),
            sender,
        };

        self.vm().emit_log(&event.encode_data(), 1);

        Ok(())
    }

    pub fn finish_edit_index(&mut self, index: U128) -> Result<(), Vec<u8>> {
        let mut storage = Keep::storage();
        let sender = self.attendee();
        storage.check_version()?;

        let vault = storage.vaults.setter(index);
        let IVault::ownerReturn { _0: owner } =
            self.static_call_ret(vault.gate_to_vault.get(), IVault::ownerCall {})?;

        if owner != self.top_level() {
            Err(b"Vault ownership must be returned")?;
        }

        let event = IGuildmaster::FinishEditIndex {
            index: index.to(),
            sender,
        };

        self.vm().emit_log(&event.encode_data(), 1);

        Ok(())
    }

    /// Submit a vote for an Index
    ///
    /// Once enough votes, Vault contract is activated.
    ///
    pub fn submit_vote(&mut self, index: U128, vote: Bytes) -> Result<(), Vec<u8>> {
        let mut storage = Keep::storage();
        storage.check_version()?;

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

    /// Update Index Quote
    ///
    /// Scan inventory assets, supply, delta, prices and liquidity and
    /// compute capacity, price and slope for an Index.
    ///
    pub fn update_index_quote(&mut self, vendor_id: U128, index_id: U128) -> Result<(), Vec<u8>> {
        let mut storage = Keep::storage();
        storage.check_version()?;

        let mut clerk_storage = ClerkStorage::storage();

        let mut vault = storage.vaults.setter(index_id);
        let vendor_quote_id = _init_vendor_quote(&mut vault, &mut clerk_storage, vendor_id);

        let account = storage.accounts.get(vendor_id);

        // Compile VIL program, which we will send to DeVIL for execution
        //
        // The program:
        //  - updates index's quote, i.e. capacity, price, slope
        //
        // Note it could be a stored procedure as program is constant for each Vault.
        //
        let update = update_quote(
            vault.assets.get().to(),
            vault.weights.get().to(),
            vendor_quote_id.to(),
            account.assets.get().to(),
            account.prices.get().to(),
            account.slopes.get().to(),
            account.liquidity.get().to(),
        );

        let clerk = storage.clerk.get();
        let num_registry = 16;
        self.update_records(clerk, update, num_registry)?;
        Ok(())
    }

    /// Update Quote for multiple Indexes
    ///
    /// This allows to update multiple Index uotes at once.
    ///
    pub fn update_multiple_index_quotes(
        &mut self,
        vendor_id: U128,
        index_ids: Vec<U128>,
    ) -> Result<(), Vec<u8>> {
        for index_id in index_ids {
            self.update_index_quote(vendor_id, index_id)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {}
