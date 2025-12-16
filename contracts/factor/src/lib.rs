// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::U128;
use amount_macros::amount;
use deli::contracts::{
    keep::{Granary, Keep},
    keep_calls::KeepCalls,
};
use icore::vil::{
    execute_buy_order::execute_buy_order, update_market_data::update_market_data,
    update_quote::update_quote,
};
use stylus_sdk::prelude::*;

#[storage]
#[entrypoint]
pub struct Factor;

#[public]
impl Factor {
    /// Submit Market Data
    ///
    /// Vendor submits Market Data using Price, Slope, Liquidity model, which is
    /// a format optimised for on-chain computation.
    ///
    /// - Price     : Micro-Price
    /// - Slope     : Price delta within N-levels (Bid + Ask)
    /// - Liquidity : Total quantitiy on N-levels (Bid + Ask)
    ///
    /// Vendor is responsible for modeling these parameters in suitable way
    /// using live Market Data.
    ///
    /// Note that it is the Vendor deciding what prices and exposure they are
    /// willing to accept, i.e. they can adjust prices, slopes and liquidity to
    /// take into account their risk factors.
    ///
    pub fn submit_market_data(
        &mut self,
        vendor_id: U128,
        asset_names: Vec<u8>,
        asset_liquidity: Vec<u8>,
        asset_prices: Vec<u8>,
        asset_slopes: Vec<u8>,
    ) -> Result<(), Vec<u8>> {
        let mut storage = Keep::storage();

        let mut account = storage.accounts.setter(vendor_id);
        account.set_only_owner(self.attendee())?;

        let gate_to_granary = storage.granary.get_granary_address();

        let asset_names_id = Granary::SCRATCH_1;
        let asset_liquidity_id = Granary::SCRATCH_2;
        let asset_prices_id = Granary::SCRATCH_3;
        let asset_slopes_id = Granary::SCRATCH_4;

        self.submit_vector_bytes(gate_to_granary, asset_names_id.to(), asset_names)?;
        self.submit_vector_bytes(gate_to_granary, asset_liquidity_id.to(), asset_liquidity)?;
        self.submit_vector_bytes(gate_to_granary, asset_prices_id.to(), asset_prices)?;
        self.submit_vector_bytes(gate_to_granary, asset_slopes_id.to(), asset_slopes)?;

        // Compile VIL program, which we will send to DeVIL for execution.
        let update = update_market_data(
            asset_names_id.to(),
            asset_prices_id.to(),
            asset_slopes_id.to(),
            asset_liquidity_id.to(),
            account.assets.get().to(),
            account.prices.get().to(),
            account.slopes.get().to(),
            account.liquidity.get().to(),
        );
        let num_registry = 16;
        self.execute_vector_program(gate_to_granary, update, num_registry)?;
        Ok(())
    }

    /// Update Index Quote
    ///
    /// Scan inventory assets, supply, delta, prices and liquidity and
    /// compute capacity, price and slope for an Index.
    ///
    pub fn update_index_quote(&mut self, vendor_id: U128, index: U128) -> Result<(), Vec<u8>> {
        let storage = Keep::storage();
        let vault = storage.vaults.get(index);
        let account = storage.accounts.get(vendor_id);
        let gate_to_granary = storage.granary.get_granary_address();

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
            vault.quote.get().to(),
            account.assets.get().to(),
            account.prices.get().to(),
            account.slopes.get().to(),
            account.liquidity.get().to(),
        );
        let num_registry = 16;
        self.execute_vector_program(gate_to_granary, update, num_registry)?;
        Ok(())
    }

    /// Update Quote for multiple Indexes
    ///
    /// This allows to update multiple Index uotes at once.
    ///
    pub fn update_multiple_index_quotes(
        &mut self,
        vendor_id: U128,
        indexes: Vec<U128>,
    ) -> Result<(), Vec<u8>> {
        for index in indexes {
            self.update_index_quote(vendor_id, index)?;
        }
        Ok(())
    }

    /// Submit BUY Index order
    ///
    /// Add collateral amount to user's order, and match for immediate execution.
    ///
    pub fn submit_buy_order(
        &mut self,
        vendor_id: U128,
        index: U128,
        collateral_amount: u128,
    ) -> Result<(), Vec<u8>> {
        let mut storage = Keep::storage();
        let mut vault = storage.vaults.setter(index);
        let account = storage.accounts.get(vendor_id);
        let gate_to_granary = storage.granary.get_granary_address();
        let user = self.attendee();

        // TODO: We need to set these up. They are from Vault and Market.
        let executed_asset_quantities_id = Granary::SCRATCH_1;
        let executed_index_quantities_id = Granary::SCRATCH_2;

        let asset_contribution_fractions_id = 0;
        let solve_quadratic_id = 0;

        let max_order_size = amount!(1000.0);

        // Allocate new Index order or extend to existing one
        let index_order_id = {
            let mut set_id = vault.orders.setter(user);
            let old_id = set_id.get();
            if old_id.is_zero() {
                let new_id = storage.granary.next_vector();
                set_id.set(new_id);
                new_id
            } else {
                old_id
            }
        };

        // Compile VIL program, which we will send to DeVIL for execution.
        //
        // The program:
        //  - updates user's order with new collateral
        //  - executes portion of the order that fits within Index capacity
        //  - updates demand and delta vectors
        //  - returns amount of collateral remaining and spent, and
        //  - Index quantity executed and remaining
        //
        let update = execute_buy_order(
            index_order_id.to(),
            collateral_amount,
            0,
            max_order_size.to_u128_raw(),
            executed_index_quantities_id.to(),
            executed_asset_quantities_id.to(),
            vault.assets.get().to(),
            vault.weights.get().to(),
            vault.quote.get().to(),
            account.assets.get().to(),
            account.supply_long.get().to(),
            account.supply_short.get().to(),
            account.demand_long.get().to(),
            account.demand_short.get().to(),
            account.delta_long.get().to(),
            account.delta_short.get().to(),
            account.margin.get().to(),
            asset_contribution_fractions_id,
            solve_quadratic_id,
        );
        let num_registry = 16;
        self.execute_vector_program(gate_to_granary, update, num_registry)?;

        // TODO: Fetch results
        // - executed and remaining Index quantity
        // - collateral remaining and spent
        // - mint token if fully executed
        let executed_asset_quantities =
            self.fetch_vector_from_granary(gate_to_granary, executed_asset_quantities_id.to())?;

        let executed_index_quantities =
            self.fetch_vector_from_granary(gate_to_granary, executed_index_quantities_id.to())?;

        let index_order = self.fetch_vector_from_granary(gate_to_granary, index_order_id.to())?;

        let _ = executed_asset_quantities;
        let _ = executed_index_quantities;
        let _ = index_order;

        // TODO: Do something with results

        Ok(())
    }
}

#[cfg(test)]
mod test {}
