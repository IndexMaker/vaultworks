// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use abacus_formulas::{
    execute_buy_order::execute_buy_order, execute_sell_order::execute_sell_order,
    execute_transfer::execute_transfer, solve_quadratic_ask::solve_quadratic_ask,
    solve_quadratic_bid::solve_quadratic_bid, update_market_data::update_market_data,
    update_quote::update_quote,
};
use alloy_primitives::{Address, U128};
use common::vector::Vector;
use common_contracts::contracts::{
    keep::{ClerkChamber, Keep, Vault},
    keep_calls::KeepCalls,
};
use stylus_sdk::prelude::*;
use vector_macros::amount_vec;

#[storage]
#[entrypoint]
pub struct Factor;

impl Factor {
    fn init_solve_quadratic_bid(&mut self, storage: &mut Keep) -> Result<U128, Vec<u8>> {
        // Q_buy = (sqrt(P^2 + 4 * S * C_buy) - P) / 2 * S
        let solve_quadratic_id = {
            let mut id = storage.solve_quadratic_bid_id.get();
            if id.is_zero() {
                id = storage.clerk_chamber.next_vector();
                let code = solve_quadratic_bid();
                self.submit_vector_bytes(storage.clerk_chamber.get_gate_address(), id.to(), code)?;
                storage.solve_quadratic_bid_id.set(id);
                id
            } else {
                id
            }
        };
        Ok(solve_quadratic_id)
    }

    fn init_solve_quadratic_ask(&mut self, storage: &mut Keep) -> Result<U128, Vec<u8>> {
        // Q_sell = (P - sqrt(P^2 - 4 * S * C_sell)) / 2 * S
        let solve_quadratic_id = {
            let mut id = storage.solve_quadratic_ask_id.get();
            if id.is_zero() {
                id = storage.clerk_chamber.next_vector();
                let code = solve_quadratic_ask();
                self.submit_vector_bytes(storage.clerk_chamber.get_gate_address(), id.to(), code)?;
                storage.solve_quadratic_ask_id.set(id);
                id
            } else {
                id
            }
        };
        Ok(solve_quadratic_id)
    }

    fn init_trader_bid(
        &mut self,
        vault: &mut Vault,
        clerk_chamber: &mut ClerkChamber,
        trader_address: Address,
    ) -> Result<U128, Vec<u8>> {
        let mut set_bid_id = vault.traders_bids.setter(trader_address);

        let bid_id = set_bid_id.get();
        if !bid_id.is_zero() {
            return Ok(bid_id);
        }

        let bid_id = clerk_chamber.next_vector();
        set_bid_id.set(bid_id);

        self.submit_vector_bytes(
            clerk_chamber.get_gate_address(),
            bid_id.to(),
            amount_vec![0, 0, 0].to_vec(),
        )?;

        if vault.traders_asks.get(trader_address).is_zero() {
            vault.traders.push(trader_address);
        }

        Ok(bid_id)
    }

    fn init_trader_ask(
        &mut self,
        vault: &mut Vault,
        clerk_chamber: &mut ClerkChamber,
        trader_address: Address,
    ) -> Result<U128, Vec<u8>> {
        let mut set_ask_id = vault.traders_asks.setter(trader_address);

        let ask_id = set_ask_id.get();
        if !ask_id.is_zero() {
            return Ok(ask_id);
        }

        let ask_id = clerk_chamber.next_vector();
        set_ask_id.set(ask_id);

        self.submit_vector_bytes(
            clerk_chamber.get_gate_address(),
            ask_id.to(),
            amount_vec![0, 0, 0].to_vec(),
        )?;

        if vault.traders_bids.get(trader_address).is_zero() {
            vault.traders.push(trader_address);
        }

        Ok(ask_id)
    }

    fn init_vendor_quote(
        &mut self,
        vault: &mut Vault,
        clerk_chamber: &mut ClerkChamber,
        vendor_id: U128,
    ) -> Result<U128, Vec<u8>> {
        let mut set_quote_id = vault.vendor_quotes.setter(vendor_id);

        let quote_id = set_quote_id.get();
        if !quote_id.is_zero() {
            return Ok(quote_id);
        }

        let quote_id = clerk_chamber.next_vector();
        set_quote_id.set(quote_id);

        self.submit_vector_bytes(
            clerk_chamber.get_gate_address(),
            quote_id.to(),
            amount_vec![0, 0, 0].to_vec(),
        )?;

        if vault.vendors_bids.get(vendor_id).is_zero()
            && vault.vendors_asks.get(vendor_id).is_zero()
        {
            vault.vendors.push(vendor_id);
        }

        Ok(quote_id)
    }

    fn init_vendor_bid(
        &mut self,
        vault: &mut Vault,
        clerk_chamber: &mut ClerkChamber,
        vendor_id: U128,
    ) -> Result<U128, Vec<u8>> {
        let mut set_bid_id = vault.vendors_bids.setter(vendor_id);

        let bid_id = set_bid_id.get();
        if !bid_id.is_zero() {
            return Ok(bid_id);
        }

        let bid_id = clerk_chamber.next_vector();
        set_bid_id.set(bid_id);

        self.submit_vector_bytes(
            clerk_chamber.get_gate_address(),
            bid_id.to(),
            amount_vec![0, 0, 0].to_vec(),
        )?;

        if vault.vendor_quotes.get(vendor_id).is_zero()
            && vault.vendors_asks.get(vendor_id).is_zero()
        {
            vault.vendors.push(vendor_id);
        }

        Ok(bid_id)
    }

    fn init_vendor_ask(
        &mut self,
        vault: &mut Vault,
        clerk_chamber: &mut ClerkChamber,
        vendor_id: U128,
    ) -> Result<U128, Vec<u8>> {
        let mut set_ask_id = vault.vendors_asks.setter(vendor_id);

        let ask_id = set_ask_id.get();
        if !ask_id.is_zero() {
            return Ok(ask_id);
        }

        let ask_id = clerk_chamber.next_vector();
        set_ask_id.set(ask_id);

        self.submit_vector_bytes(
            clerk_chamber.get_gate_address(),
            ask_id.to(),
            amount_vec![0, 0, 0].to_vec(),
        )?;

        if vault.vendor_quotes.get(vendor_id).is_zero()
            && vault.vendors_bids.get(vendor_id).is_zero()
        {
            vault.vendors.push(vendor_id);
        }

        Ok(ask_id)
    }

    fn init_total_bid(
        &mut self,
        vault: &mut Vault,
        clerk_chamber: &mut ClerkChamber,
    ) -> Result<U128, Vec<u8>> {
        let bid_id = vault.total_bid.get();
        if !bid_id.is_zero() {
            return Ok(bid_id);
        }

        let bid_id = clerk_chamber.next_vector();
        vault.total_bid.set(bid_id);

        self.submit_vector_bytes(
            clerk_chamber.get_gate_address(),
            bid_id.to(),
            amount_vec![0, 0, 0].to_vec(),
        )?;

        Ok(bid_id)
    }

    fn init_total_ask(
        &mut self,
        vault: &mut Vault,
        clerk_chamber: &mut ClerkChamber,
    ) -> Result<U128, Vec<u8>> {
        let ask_id = vault.total_ask.get();
        if !ask_id.is_zero() {
            return Ok(ask_id);
        }

        let ask_id = clerk_chamber.next_vector();
        vault.total_ask.set(ask_id);

        self.submit_vector_bytes(
            clerk_chamber.get_gate_address(),
            ask_id.to(),
            amount_vec![0, 0, 0].to_vec(),
        )?;

        Ok(ask_id)
    }
}

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

        let gate_to_clerk_chamber = storage.clerk_chamber.get_gate_address();

        let asset_names_id = ClerkChamber::SCRATCH_1;
        let asset_liquidity_id = ClerkChamber::SCRATCH_2;
        let asset_prices_id = ClerkChamber::SCRATCH_3;
        let asset_slopes_id = ClerkChamber::SCRATCH_4;

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
    pub fn update_index_quote(&mut self, vendor_id: U128, index_id: U128) -> Result<(), Vec<u8>> {
        let mut storage = Keep::storage();
        let mut vault = storage.vaults.setter(index_id);

        let vendor_quote_id =
            self.init_vendor_quote(&mut vault, &mut storage.clerk_chamber, vendor_id)?;

        let account = storage.accounts.get(vendor_id);
        let gate_to_clerk_chamber = storage.clerk_chamber.get_gate_address();

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
        index_ids: Vec<U128>,
    ) -> Result<(), Vec<u8>> {
        for index_id in index_ids {
            self.update_index_quote(vendor_id, index_id)?;
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
        index_id: U128,
        collateral_added: u128,
        collateral_removed: u128,
        max_order_size: u128,
        asset_contribution_fractions: Vec<u8>,
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), Vec<u8>> {
        let mut storage = Keep::storage();

        // Allocate Quadratic Solver
        let solve_quadratic_id = self.init_solve_quadratic_bid(&mut storage)?;

        let mut vault = storage.vaults.setter(index_id);
        let trader_address = self.attendee();

        // Allocate new Index order or get existing one
        let index_order_id =
            self.init_trader_bid(&mut vault, &mut storage.clerk_chamber, trader_address)?;

        let vendor_quote_id =
            self.init_vendor_quote(&mut vault, &mut storage.clerk_chamber, vendor_id)?;

        let vendor_order_id =
            self.init_vendor_bid(&mut vault, &mut storage.clerk_chamber, vendor_id)?;

        let total_order_id = self.init_total_bid(&mut vault, &mut storage.clerk_chamber)?;

        let account = storage.accounts.get(vendor_id);
        let gate_to_clerk_chamber = storage.clerk_chamber.get_gate_address();

        let asset_contribution_fractions_id = ClerkChamber::SCRATCH_1;
        self.submit_vector_bytes(
            gate_to_clerk_chamber,
            asset_contribution_fractions_id.to(),
            asset_contribution_fractions,
        )?;

        let executed_asset_quantities_id = ClerkChamber::SCRATCH_2;
        let executed_index_quantities_id = ClerkChamber::SCRATCH_3;

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
            index_order_id.to(), // single trader orders aggregated per vault (we don't store individual orders)
            vendor_order_id.to(),
            total_order_id.to(),
            collateral_added,
            collateral_removed,
            max_order_size,
            executed_index_quantities_id.to(),
            executed_asset_quantities_id.to(),
            vault.assets.get().to(),
            vault.weights.get().to(),
            vendor_quote_id.to(),
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
        let num_registry = 22;
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

    pub fn submit_sell_order(
        &mut self,
        vendor_id: U128,
        index_id: U128,
        collateral_added: u128,
        collateral_removed: u128,
        max_order_size: u128,
        asset_contribution_fractions: Vec<u8>,
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), Vec<u8>> {
        let mut storage = Keep::storage();

        // Allocate Quadratic Solver
        let solve_quadratic_id = self.init_solve_quadratic_ask(&mut storage)?;

        let mut vault = storage.vaults.setter(index_id);
        let trader_address = self.attendee();

        // Allocate new Index order or get existing one
        let index_order_id =
            self.init_trader_ask(&mut vault, &mut storage.clerk_chamber, trader_address)?;

        let vendor_quote_id =
            self.init_vendor_quote(&mut vault, &mut storage.clerk_chamber, vendor_id)?;

        let vendor_order_id =
            self.init_vendor_ask(&mut vault, &mut storage.clerk_chamber, vendor_id)?;

        let total_order_id = self.init_total_ask(&mut vault, &mut storage.clerk_chamber)?;

        let account = storage.accounts.get(vendor_id);
        let gate_to_clerk_chamber = storage.clerk_chamber.get_gate_address();

        let asset_contribution_fractions_id = ClerkChamber::SCRATCH_1;
        self.submit_vector_bytes(
            gate_to_clerk_chamber,
            asset_contribution_fractions_id.to(),
            asset_contribution_fractions,
        )?;

        let executed_asset_quantities_id = ClerkChamber::SCRATCH_2;
        let executed_index_quantities_id = ClerkChamber::SCRATCH_3;

        // Compile VIL program, which we will send to DeVIL for execution.
        //
        // The program:
        //  - updates user's order with new collateral
        //  - executes portion of the order that fits within Index capacity
        //  - updates demand and delta vectors
        //  - returns amount of collateral remaining and spent, and
        //  - Index quantity executed and remaining
        //
        let update = execute_sell_order(
            index_order_id.to(), // single trader orders aggregated per vault (we don't store individual orders)
            vendor_order_id.to(),
            total_order_id.to(),
            collateral_added,
            collateral_removed,
            max_order_size,
            executed_index_quantities_id.to(),
            executed_asset_quantities_id.to(),
            vault.assets.get().to(),
            vault.weights.get().to(),
            vendor_quote_id.to(),
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
        let num_registry = 22;
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

    pub fn submit_rebalance_order(
        &mut self,
        vendor_id: U128,
        new_assets: Vec<u8>,
        new_weigthts: Vec<u8>,
    ) -> Result<(), Vec<u8>> {
        //
        // This needs to:
        //  - compute rebalance_weights_long = max(0, weights - new_weights) -- assets long in inventory (sell them)
        //  - compute rebalance_weights_short = max(0, new_weights - weights) -- assets short in inventory (buy more)
        //
        Err(b"Not implemented yet".into())
    }

    pub fn submit_transfer_order(
        &mut self,
        index_id: U128,
        receiver: Address,
        amount: u128,
    ) -> Result<(), Vec<u8>> {
        let mut storage = Keep::storage();
        let mut vault = storage.vaults.setter(index_id);
        let sender = self.attendee();

        let sender_bid_id = self.init_trader_bid(&mut vault, &mut storage.clerk_chamber, sender)?;
        let sender_ask_id = self.init_trader_ask(&mut vault, &mut storage.clerk_chamber, sender)?;
        let receiver_bid_id =
            self.init_trader_bid(&mut vault, &mut storage.clerk_chamber, receiver)?;

        let gate_to_clerk_chamber = storage.clerk_chamber.get_gate_address();

        // TODO: query balance before transaction to check if sufficient instead of 
        // relying on execute_transfer() to fail in such case, so that we have friendly
        // error message.
        let (sender_bid_bytes, sender_ask_bytes) = self.get_trader_order(index_id, sender)?;
        let sender_bid = Vector::from_vec(sender_bid_bytes);
        let sender_ask = Vector::from_vec(sender_ask_bytes);
        let sender_minted = sender_bid.data[2];
        let sender_redeem = sender_ask.data[0];
        let sender_balance = sender_minted.checked_sub(sender_redeem).ok_or_else(|| b"Unexpected minted < redeem")?;
        if sender_balance.to_u128_raw() < amount {
            Err(b"Insufficient amount of Index token")?;
        }

        let update = execute_transfer(
            sender_bid_id.to(),
            sender_ask_id.to(),
            receiver_bid_id.to(),
            amount,
        );

        let num_registry = 6;
        self.execute_vector_program(gate_to_clerk_chamber, update, num_registry)?;

        Ok(())
    }

    //
    // Query methods
    //

    // ERC-20 & ERC-4626
    // @{
    // pub fn get_trader_balance() {}  // bid.ITP_minted - ask.ITP_rem ; trader's balance excludes part they submitted in sell order
    // pub fn get_trader_assets() {}   // for i..n_chunks: balance_i * (quote.P + balance_i * quote.S) ; chunk balance by max-order-size
    // pub fn get_total_supply() {}    // total.bid.ITP_minted - total.ask.ITP_burned ; total supply includes all ITP that is not burned
    // pub fn get_total_assets() {}    // for i..n_chunks: supply_i * (quote.P + supply_i * quote.S) ; chunk supply by max-order-size
    // pub fn convert_to_shares() {}   // solve quadratic for max-order-size (n_chunks - 1) times and once for remainder
    // pub fn convert_to_assets() {}   // for i..n_chunks: amount_i * (quote.P + amount_i * quote.S) ; chunk amount by max-order-size
    // pub fn preview_deposit() {}     // same(ish) as convert_to_shares()
    // pub fn preview_redeem() {}      // same(ish) as convert_to_redeem()
    // @}

    pub fn get_market_data(&self, vendor_id: U128) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), Vec<u8>> {
        let storage = Keep::storage();
        let account = storage.accounts.get(vendor_id);
        let gate_to_clerk_chamber = storage.clerk_chamber.get_gate_address();

        let liquidity =
            self.fetch_vector_bytes(gate_to_clerk_chamber, account.liquidity.get().to())?;

        let prices = self.fetch_vector_bytes(gate_to_clerk_chamber, account.prices.get().to())?;
        let slopes = self.fetch_vector_bytes(gate_to_clerk_chamber, account.slopes.get().to())?;

        Ok((liquidity, prices, slopes))
    }

    pub fn get_index_assets(&self, index_id: U128) -> Result<Vec<u8>, Vec<u8>> {
        let storage = Keep::storage();
        let vault = storage.vaults.get(index_id);
        let gate_to_clerk_chamber = storage.clerk_chamber.get_gate_address();
        let data = self.fetch_vector_bytes(gate_to_clerk_chamber, vault.assets.get().to())?;
        Ok(data)
    }

    pub fn get_index_weights(&self, index_id: U128) -> Result<Vec<u8>, Vec<u8>> {
        let storage = Keep::storage();
        let vault = storage.vaults.get(index_id);
        let gate_to_clerk_chamber = storage.clerk_chamber.get_gate_address();
        let data = self.fetch_vector_bytes(gate_to_clerk_chamber, vault.weights.get().to())?;
        Ok(data)
    }

    pub fn get_index_quote(&self, index_id: U128, vendor_id: U128) -> Result<Vec<u8>, Vec<u8>> {
        let storage = Keep::storage();
        let vault = storage.vaults.get(index_id);
        let gate_to_clerk_chamber = storage.clerk_chamber.get_gate_address();
        let quote_id = vault.vendor_quotes.get(vendor_id);
        if quote_id.is_zero() {
            Err(b"No such quote")?;
        }
        let data = self.fetch_vector_bytes(gate_to_clerk_chamber, quote_id.to())?;
        Ok(data)
    }

    pub fn get_trader_order(
        &self,
        index_id: U128,
        trader: Address,
    ) -> Result<(Vec<u8>, Vec<u8>), Vec<u8>> {
        let storage = Keep::storage();
        let vault = storage.vaults.get(index_id);
        let gate_to_clerk_chamber = storage.clerk_chamber.get_gate_address();

        let bid_id = vault.traders_bids.get(trader);

        let bid = if !bid_id.is_zero() {
            self.fetch_vector_bytes(gate_to_clerk_chamber, bid_id.to())?
        } else {
            amount_vec!(0, 0, 0).to_vec()
        };

        let ask_id = vault.traders_asks.get(trader);
        let ask = if !ask_id.is_zero() {
            self.fetch_vector_bytes(gate_to_clerk_chamber, ask_id.to())?
        } else {
            amount_vec!(0, 0, 0).to_vec()
        };

        Ok((bid, ask))
    }

    pub fn get_trader_count(&self, index_id: U128) -> Result<U128, Vec<u8>> {
        let storage = Keep::storage();
        let vault = storage.vaults.get(index_id);

        let result = U128::from(vault.traders.len());
        Ok(result)
    }

    pub fn get_trader_order_at(
        &self,
        index_id: U128,
        offset: u128,
    ) -> Result<(Address, Vec<u8>, Vec<u8>), Vec<u8>> {
        let storage = Keep::storage();
        let vault = storage.vaults.get(index_id);
        let gate_to_clerk_chamber = storage.clerk_chamber.get_gate_address();

        if let Some(address) = vault.traders.get(offset) {
            let bid_id = vault.traders_bids.get(address);

            let bid = if !bid_id.is_zero() {
                self.fetch_vector_bytes(gate_to_clerk_chamber, bid_id.to())?
            } else {
                amount_vec!(0, 0, 0).to_vec()
            };

            let ask_id = vault.traders_asks.get(address);
            let ask = if !ask_id.is_zero() {
                self.fetch_vector_bytes(gate_to_clerk_chamber, ask_id.to())?
            } else {
                amount_vec!(0, 0, 0).to_vec()
            };

            Ok((address, bid, ask))
        } else {
            Err(b"No such order".into())
        }
    }

    pub fn get_vendor_order(
        &self,
        index_id: U128,
        vendor_id: U128,
    ) -> Result<(Vec<u8>, Vec<u8>), Vec<u8>> {
        let storage = Keep::storage();
        let vault = storage.vaults.get(index_id);
        let gate_to_clerk_chamber = storage.clerk_chamber.get_gate_address();

        let bid_id = vault.vendors_bids.get(vendor_id);

        let bid = if !bid_id.is_zero() {
            self.fetch_vector_bytes(gate_to_clerk_chamber, bid_id.to())?
        } else {
            amount_vec!(0, 0, 0).to_vec()
        };

        let ask_id = vault.vendors_asks.get(vendor_id);
        let ask = if !ask_id.is_zero() {
            self.fetch_vector_bytes(gate_to_clerk_chamber, ask_id.to())?
        } else {
            amount_vec!(0, 0, 0).to_vec()
        };

        Ok((bid, ask))
    }

    pub fn get_vendor_count(&self, index_id: U128) -> Result<U128, Vec<u8>> {
        let storage = Keep::storage();
        let vault = storage.vaults.get(index_id);

        let result = U128::from(vault.vendors.len());
        Ok(result)
    }

    pub fn get_vendor_order_at(
        &self,
        index_id: U128,
        offset: u128,
    ) -> Result<(U128, Vec<u8>, Vec<u8>), Vec<u8>> {
        let storage = Keep::storage();
        let vault = storage.vaults.get(index_id);
        let gate_to_clerk_chamber = storage.clerk_chamber.get_gate_address();

        if let Some(vendor_id) = vault.vendors.get(offset) {
            let bid_id = vault.vendors_bids.get(vendor_id);

            let bid = if !bid_id.is_zero() {
                self.fetch_vector_bytes(gate_to_clerk_chamber, bid_id.to())?
            } else {
                amount_vec!(0, 0, 0).to_vec()
            };

            let ask_id = vault.vendors_asks.get(vendor_id);
            let ask = if !ask_id.is_zero() {
                self.fetch_vector_bytes(gate_to_clerk_chamber, ask_id.to())?
            } else {
                amount_vec!(0, 0, 0).to_vec()
            };

            Ok((vendor_id, bid, ask))
        } else {
            Err(b"No such order".into())
        }
    }

    pub fn get_total_order(&self, index_id: U128) -> Result<(Vec<u8>, Vec<u8>), Vec<u8>> {
        let storage = Keep::storage();
        let vault = storage.vaults.get(index_id);
        let gate_to_clerk_chamber = storage.clerk_chamber.get_gate_address();

        let bid_id = vault.total_bid.get();

        let bid = if !bid_id.is_zero() {
            self.fetch_vector_bytes(gate_to_clerk_chamber, bid_id.to())?
        } else {
            amount_vec!(0, 0, 0).to_vec()
        };

        let ask_id = vault.total_ask.get();
        let ask = if !ask_id.is_zero() {
            self.fetch_vector_bytes(gate_to_clerk_chamber, ask_id.to())?
        } else {
            amount_vec!(0, 0, 0).to_vec()
        };

        Ok((bid, ask))
    }
}
