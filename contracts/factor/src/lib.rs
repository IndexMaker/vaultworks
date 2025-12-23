// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use abacus_formulas::{
    execute_buy_order::execute_buy_order, solve_quadratic::solve_quadratic,
    update_market_data::update_market_data, update_quote::update_quote,
};
use alloy_primitives::{Address, U128};
use common::vector::Vector;
use common_contracts::contracts::{
    keep::{Clerk, Keep},
    keep_calls::KeepCalls,
};
use stylus_sdk::prelude::*;
use vector_macros::amount_vec;

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

        let account = storage.accounts.setter(vendor_id);
        account.only_owner(self.attendee())?;

        let gate_to_clerk_chamber = storage.clerk.get_clerk_address();

        let asset_names_id = Clerk::SCRATCH_1;
        let asset_liquidity_id = Clerk::SCRATCH_2;
        let asset_prices_id = Clerk::SCRATCH_3;
        let asset_slopes_id = Clerk::SCRATCH_4;

        self.submit_vector_bytes(gate_to_clerk_chamber, asset_names_id.to(), asset_names)?;
        self.submit_vector_bytes(
            gate_to_clerk_chamber,
            asset_liquidity_id.to(),
            asset_liquidity,
        )?;
        self.submit_vector_bytes(gate_to_clerk_chamber, asset_prices_id.to(), asset_prices)?;
        self.submit_vector_bytes(gate_to_clerk_chamber, asset_slopes_id.to(), asset_slopes)?;

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
        self.execute_vector_program(gate_to_clerk_chamber, update, num_registry)?;
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
        let gate_to_clerk_chamber = storage.clerk.get_clerk_address();

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
        self.execute_vector_program(gate_to_clerk_chamber, update, num_registry)?;
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
        collateral_added: u128,
        collateral_removed: u128,
        max_order_size: u128,
        asset_contribution_fractions: Vec<u8>,
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), Vec<u8>> {
        let mut storage = Keep::storage();
        let mut vault = storage.vaults.setter(index);
        let account = storage.accounts.get(vendor_id);
        let gate_to_clerk_chamber = storage.clerk.get_clerk_address();
        let user = self.attendee();

        let asset_contribution_fractions_id = Clerk::SCRATCH_1;
        self.submit_vector_bytes(
            gate_to_clerk_chamber,
            asset_contribution_fractions_id.to(),
            asset_contribution_fractions,
        )?;

        let executed_asset_quantities_id = Clerk::SCRATCH_2;
        let executed_index_quantities_id = Clerk::SCRATCH_3;

        let solve_quadratic_id = {
            let mut id = storage.solve_quadratic_id.get();
            if id.is_zero() {
                id = storage.clerk.next_vector();
                let code = solve_quadratic();
                self.submit_vector_bytes(gate_to_clerk_chamber, id.to(), code)?;
                storage.solve_quadratic_id.set(id);
                id
            } else {
                id
            }
        };

        // Allocate new Index order or extend to existing one
        let index_order_id = {
            let mut set_id = vault.orders.setter(user);
            let old_id = set_id.get();
            if old_id.is_zero() {
                let new_id = storage.clerk.next_vector();
                set_id.set(new_id);
                self.submit_vector_bytes(
                    gate_to_clerk_chamber,
                    new_id.to(),
                    amount_vec![0, 0, 0].to_vec(),
                )?;
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
            collateral_added,
            collateral_removed,
            max_order_size,
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
            asset_contribution_fractions_id.to(),
            solve_quadratic_id.to(),
        );
        let num_registry = 16;
        self.execute_vector_program(gate_to_clerk_chamber, update, num_registry)?;

        // TODO: Fetch results
        // - executed and remaining Index quantity
        // - collateral remaining and spent
        // - mint token if fully executed
        let executed_asset_quantities =
            self.fetch_vector_from_clerk(gate_to_clerk_chamber, executed_asset_quantities_id.to())?;

        let executed_index_quantities =
            self.fetch_vector_from_clerk(gate_to_clerk_chamber, executed_index_quantities_id.to())?;

        let index_order =
            self.fetch_vector_from_clerk(gate_to_clerk_chamber, index_order_id.to())?;

        Ok((
            index_order.to_vec(),
            executed_index_quantities.to_vec(),
            executed_asset_quantities.to_vec(),
        ))
    }

    pub fn fetch_market_data(
        &self,
        vendor_id: U128,
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), Vec<u8>> {
        let storage = Keep::storage();
        let account = storage.accounts.get(vendor_id);
        let gate_to_clerk_chamber = storage.clerk.get_clerk_address();

        let liquidity =
            self.fetch_vector_bytes(gate_to_clerk_chamber, account.liquidity.get().to())?;
        let prices = self.fetch_vector_bytes(gate_to_clerk_chamber, account.prices.get().to())?;
        let slopes = self.fetch_vector_bytes(gate_to_clerk_chamber, account.slopes.get().to())?;

        Ok((liquidity, prices, slopes))
    }

    pub fn fetch_index_quote(&self, index: U128) -> Result<Vec<u8>, Vec<u8>> {
        let storage = Keep::storage();
        let vault = storage.vaults.get(index);
        let gate_to_clerk_chamber = storage.clerk.get_clerk_address();

        let quote = self.fetch_vector_bytes(gate_to_clerk_chamber, vault.quote.get().to())?;

        Ok(quote)
    }

    pub fn get_order_count(&self, index: U128) -> Result<U128, Vec<u8>> {
        let storage = Keep::storage();
        let vault = storage.vaults.get(index);
        
        Ok(U128::ZERO)
    }

    pub fn get_order(&self, index: U128, offset: U128) -> Result<(Address, Vec<u8>), Vec<u8>> {
        let storage = Keep::storage();
        let vault = storage.vaults.get(index);
        
        Ok((Address::ZERO, vec![]))
    }
}

#[cfg(test)]
mod test {}
