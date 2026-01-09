// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use abacus_formulas::{
    add_market_assets::add_market_assets, create_market::create_market,
    update_margin::update_margin, update_supply::update_supply,
};
use alloy_primitives::U128;
use common_contracts::contracts::{
    clerk::{ClerkStorage, SCRATCH_1, SCRATCH_2, SCRATCH_3},
    keep::Keep,
    keep_calls::KeepCalls,
};
use stylus_sdk::{abi::Bytes, prelude::*};

#[storage]
#[entrypoint]
pub struct Banker;

#[public]
impl Banker {
    /// Submit list of all available assets
    ///
    /// Full list of assets must be submitted prior any Index or Market
    /// operation. List can be updated using multiple submit_assets call.
    ///
    /// Note that the new list must be a superset of current list or call will
    /// fail. Delisting assets is not possible. To support delisting we would
    /// need to have a process in place to first reduce supply and delta for the
    /// delisted assets to zero, and then check they are zero using JFLT, VMAX,
    /// and SUB operations, i.e. JFLT delisted assets, VMAX to find any non-zero
    /// value, and SUB to fail if non-zero value is found.
    ///
    pub fn submit_assets(
        &mut self,
        vendor_id: U128,
        market_asset_names: Bytes,
    ) -> Result<(), Vec<u8>> {
        let mut storage = Keep::storage();
        storage.check_version()?;

        let mut account = storage.accounts.setter(vendor_id);
        if account.has_owner() {
            account.only_owner(self.attendee())?;

            let new_market_asset_names_id = SCRATCH_1;

            let mut clerk_storage = ClerkStorage::storage();
            clerk_storage.store_bytes(new_market_asset_names_id, market_asset_names);

            // Compile VIL program, which we will send to DeVIL for execution.
            //
            // The program:
            // - updates market asset names
            // - extends supply, demand, and delta vectors
            // - extends prices, slopes, liquidity vectors
            //
            let update = add_market_assets(
                new_market_asset_names_id.to(),
                account.assets.get().to(),
                account.prices.get().to(),
                account.slopes.get().to(),
                account.liquidity.get().to(),
                account.supply_long.get().to(),
                account.supply_short.get().to(),
                account.demand_long.get().to(),
                account.demand_short.get().to(),
                account.delta_long.get().to(),
                account.delta_short.get().to(),
                account.margin.get().to(),
            );
            let clerk = storage.clerk.get();
            let num_registry = 16;
            self.update_records(clerk, update, num_registry)?;
        } else {
            account.set_owner(self.attendee())?;

            let new_market_asset_names_id = SCRATCH_1;

            let mut clerk_storage = ClerkStorage::storage();
            clerk_storage.store_bytes(new_market_asset_names_id, market_asset_names);

            account.assets.set(clerk_storage.next_vector());
            account.prices.set(clerk_storage.next_vector());
            account.slopes.set(clerk_storage.next_vector());
            account.liquidity.set(clerk_storage.next_vector());
            account.supply_long.set(clerk_storage.next_vector());
            account.supply_short.set(clerk_storage.next_vector());
            account.demand_long.set(clerk_storage.next_vector());
            account.demand_short.set(clerk_storage.next_vector());
            account.delta_long.set(clerk_storage.next_vector());
            account.delta_short.set(clerk_storage.next_vector());
            account.margin.set(clerk_storage.next_vector());

            // Compile VIL program, which we will send to DeVIL for execution.
            let update = create_market(
                new_market_asset_names_id.to(),
                account.assets.get().to(),
                account.prices.get().to(),
                account.slopes.get().to(),
                account.liquidity.get().to(),
                account.supply_long.get().to(),
                account.supply_short.get().to(),
                account.demand_long.get().to(),
                account.demand_short.get().to(),
                account.delta_long.get().to(),
                account.delta_short.get().to(),
                account.margin.get().to(),
            );
            let clerk = storage.clerk.get();
            let num_registry = 16;
            self.update_records(clerk, update, num_registry)?;
        }

        Ok(())
    }

    /// Submit Margin
    ///
    /// Vendor submits Margin, which limits how much of each asset we can
    /// allocate to new Index orders.
    ///
    /// Asset Capacity = MIN(Market Liquidity, Margin - MAX(Delta Short, Delta Long))
    ///
    /// Index Capacity = VMIN(Asset Capacity / Asset Weight)
    ///
    pub fn submit_margin(
        &mut self,
        vendor_id: U128,
        asset_names: Bytes,
        asset_margin: Bytes,
    ) -> Result<(), Vec<u8>> {
        let mut storage = Keep::storage();
        storage.check_version()?;

        let account = storage.accounts.setter(vendor_id);
        account.only_owner(self.attendee())?;

        let new_asset_names_id = SCRATCH_1;
        let new_asset_margin_id = SCRATCH_2;

        let mut clerk_storage = ClerkStorage::storage();
        clerk_storage.store_bytes(new_asset_names_id, asset_names);
        clerk_storage.store_bytes(new_asset_margin_id, asset_margin);

        // Compile VIL program, which we will send to DeVIL for execution.
        //
        // The program:
        // - updates margin by overwriting with supplied values
        //
        let update = update_margin(
            new_asset_names_id.to(),
            new_asset_margin_id.to(),
            account.assets.get().to(),
            account.margin.get().to(),
        );
        let clerk = storage.clerk.get();
        let num_registry = 16;
        self.update_records(clerk, update, num_registry)?;
        Ok(())
    }

    /// Submit supply
    ///
    /// Vendor submits new supply of assets. This new supply is an absolute
    /// quantity of assets and not delta.  However the supply is a sub-set of
    /// all assets stored in supply vector, so that Vendor does not need to send
    /// whole supply all the time, and only quantities of assets that have
    /// changed, e.g. as a result of fill. Vendor would accumulate fills over
    /// time period so that it doesn't call submit_supply() too often to save on
    /// gas, and in that time period Vendor would accumulate several fills for
    /// various assets, and absolute quantities of those assets after applying
    /// those fills would be submitted.
    ///
    /// Note that it is Vendor deciding how much of their internal inventory
    /// they are exposing to our transactions.
    ///
    pub fn submit_supply(
        &mut self,
        vendor_id: U128,
        asset_names: Bytes,
        asset_quantities_short: Bytes,
        asset_quantities_long: Bytes,
    ) -> Result<(), Vec<u8>> {
        let mut storage = Keep::storage();
        storage.check_version()?;

        let account = storage.accounts.setter(vendor_id);
        account.only_owner(self.attendee())?;

        let new_asset_names_id = SCRATCH_1;
        let new_asset_quantities_short_id = SCRATCH_2;
        let new_asset_quantities_long_id = SCRATCH_3;

        let mut clerk_storage = ClerkStorage::storage();
        clerk_storage.store_bytes(new_asset_names_id, asset_names);
        clerk_storage.store_bytes(new_asset_quantities_short_id, asset_quantities_short);
        clerk_storage.store_bytes(new_asset_quantities_long_id, asset_quantities_long);

        // Compile VIL program, which we will send to DeVIL for execution.
        //
        // The program:
        // - updates supply long and short by overwriting with supplied values
        // - computes delta long and short
        //
        let update = update_supply(
            new_asset_names_id.to(),
            new_asset_quantities_short_id.to(),
            new_asset_quantities_long_id.to(),
            account.assets.get().to(),
            account.supply_long.get().to(),
            account.supply_short.get().to(),
            account.demand_long.get().to(),
            account.demand_short.get().to(),
            account.delta_long.get().to(),
            account.delta_short.get().to(),
        );
        let clerk = storage.clerk.get();
        let num_registry = 16;
        self.update_records(clerk, update, num_registry)?;
        Ok(())
    }
}