// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use abacus_formulas::{execute_rebalance::execute_rebalance, update_rebalance::update_rebalance};
use alloc::vec::Vec;

use alloy_primitives::{Address, U128};
use common::{amount::Amount, labels::Labels, vector::Vector};
use common_contracts::{
    contracts::{clerk::{ClerkStorage, SCRATCH_1, SCRATCH_2}, keep::Keep, keep_calls::KeepCalls},
    interfaces::guildmaster::IGuildmaster,
};
use stylus_sdk::{abi::Bytes, prelude::*, stylus_core};

#[storage]
#[entrypoint]
pub struct Alchemist;

#[public]
impl Alchemist {
    pub fn submit_asset_weights(
        &mut self,
        index_id: U128,
        asset_names: Bytes,
        asset_weights: Bytes,
    ) -> Result<(), Vec<u8>> {
        if index_id.is_zero() {
            Err(b"Index ID cannot be zero")?;
        }
        let num_assets =
            Labels::len_from_vec(&asset_names).ok_or_else(|| b"Invalid Asset Names")?;

        if num_assets
            != Vector::len_from_vec(&asset_weights).ok_or_else(|| b"Invalid Asset Weights")?
        {
            Err(b"Asset Names and Asset Weights are not aligned")?;
        }

        let mut storage = Keep::storage();
        let sender = self.attendee();
        storage.check_version()?;

        let mut vault = storage.vaults.setter(index_id);
        vault.only_initialized()?;

        if vault.assets.get().is_zero() {
            let mut clerk_storage = ClerkStorage::storage();
            let asset_names_id = clerk_storage.next_vector();
            let asset_weights_id = clerk_storage.next_vector();

            clerk_storage.store_bytes(asset_names_id, asset_names);
            clerk_storage.store_bytes(asset_weights_id, asset_weights);

            vault.assets.set(asset_names_id);
            vault.weights.set(asset_weights_id);
        } else {
            // TODO:
            // - initialize all uninitialized vectors
            // - store new weights and names

            let new_asset_names = SCRATCH_1;
            let new_asset_weights = SCRATCH_2;

            let update = update_rebalance(
                vault.total_bid.get().to(),
                vault.total_ask.get().to(),
                vault.assets.get().to(),
                vault.weights.get().to(),
                new_asset_names.to(),
                new_asset_weights.to(),
                vault.rebalance_assets.get().to(),
                vault.rebalance_weights_long.get().to(),
                vault.rebalance_weights_short.get().to(),
            );
        }

        stylus_core::log(
            self.vm(),
            IGuildmaster::IndexWeightsUpdated {
                index_id: index_id.to(),
                sender,
            },
        );

        Ok(())
    }

    pub fn process_pending_rebalance(
        &mut self,
        vendor_id: U128,
        index_id: U128,
        capacity_factor: u128,
    ) -> Result<Vec<Bytes>, Vec<u8>> {
        if vendor_id.is_zero() {
            Err(b"Vendor ID cannot be zero")?;
        }
        if index_id.is_zero() {
            Err(b"Index ID cannot be zero")?;
        }

        let capacity_factor_f = Amount::from_u128_raw(capacity_factor);
        if Amount::ONE.is_less_than(&capacity_factor_f) {
            Err(b"Invalid capactiy factor")?;
        }

        // let update = execute_rebalance(
        //     capacity_factor,
        //     executed_assets_long_id,
        //     executed_assets_short_id,
        //     rebalance_asset_names_id,
        //     rebalance_weights_long_id,
        //     rebalance_weights_short_id,
        //     market_asset_names_id,
        //     supply_long_id,
        //     supply_short_id,
        //     demand_long_id,
        //     demand_short_id,
        //     delta_long_id,
        //     delta_short_id,
        //     margin_id,
        //     asset_liquidity_id,
        // );

        Ok(vec![])
    }
}
