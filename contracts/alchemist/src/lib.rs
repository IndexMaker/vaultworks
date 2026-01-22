// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use abacus_formulas::{execute_rebalance::execute_rebalance, update_rebalance::update_rebalance};
use alloc::vec::Vec;

use alloy_primitives::U128;
use common::{amount::Amount, labels::Labels, vector::Vector};
use common_contracts::{
    contracts::{
        clerk::{ClerkStorage, SCRATCH_1, SCRATCH_2},
        clerk_util::{new_labels, new_labels_empty, new_vector, new_vector_3z, new_vector_bytes, new_vector_empty},
        keep::Keep,
        keep_calls::KeepCalls,
    },
    interfaces::alchemist::IAlchemist,
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

        let mut clerk_storage = ClerkStorage::storage();

        if vault.assets.get().is_zero() {
            let asset_names_id = new_vector_bytes(&mut clerk_storage, asset_names);
            let asset_weights_id = new_vector_bytes(&mut clerk_storage, asset_weights);

            let rebalance_assets_id = new_labels_empty(&mut clerk_storage);
            let rebalance_weights_long_id = new_vector_empty(&mut clerk_storage);
            let rebalance_weights_short_id = new_vector_empty(&mut clerk_storage);

            let total_bid_id = new_vector_3z(&mut clerk_storage);
            let total_ask_id = new_vector_3z(&mut clerk_storage);

            vault.assets.set(asset_names_id);
            vault.weights.set(asset_weights_id);

            vault.rebalance_assets.set(rebalance_assets_id);
            vault.rebalance_weights_long.set(rebalance_weights_long_id);
            vault
                .rebalance_weights_short
                .set(rebalance_weights_short_id);

            vault.total_bid.set(total_bid_id);
            vault.total_ask.set(total_ask_id);
        } else {
            let new_asset_names = SCRATCH_1;
            let new_asset_weights = SCRATCH_2;

            clerk_storage.store_bytes(new_asset_names, asset_names);
            clerk_storage.store_bytes(new_asset_weights, asset_weights);

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

            let clerk = storage.clerk.get();
            let num_registry = 8;
            self.update_records(clerk, update?, num_registry)?;
        }

        stylus_core::log(
            self.vm(),
            IAlchemist::IndexWeightsUpdated {
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
        let mut storage = Keep::storage();
        storage.check_version()?;

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

        let vault = storage.vaults.setter(index_id);
        vault.only_initialized()?;

        let account = storage.accounts.get(vendor_id);

        let executed_assets_long_id = SCRATCH_1;
        let executed_assets_short_id = SCRATCH_2;

        let update = execute_rebalance(
            capacity_factor,
            executed_assets_long_id.to(),
            executed_assets_short_id.to(),
            vault.rebalance_assets.get().to(),
            vault.rebalance_weights_long.get().to(),
            vault.rebalance_weights_short.get().to(),
            account.assets.get().to(),
            account.supply_long.get().to(),
            account.supply_short.get().to(),
            account.demand_long.get().to(),
            account.demand_short.get().to(),
            account.delta_long.get().to(),
            account.delta_short.get().to(),
            account.margin.get().to(),
            account.liquidity.get().to(),
        );

        let clerk = storage.clerk.get();
        let num_registry = 12;
        self.update_records(clerk, update?, num_registry)?;

        let clerk_storage = ClerkStorage::storage();

        let executed_assets_long = clerk_storage
            .fetch_bytes(executed_assets_long_id)
            .ok_or_else(|| b"Executed asset quantities (long) not set")?;

        let executed_assets_short = clerk_storage
            .fetch_bytes(executed_assets_short_id)
            .ok_or_else(|| b"Executed asset quantities (short) not set")?;

        Ok(vec![
            executed_assets_long.into(),
            executed_assets_short.into(),
        ])
    }
}
