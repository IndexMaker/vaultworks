// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use abacus_formulas::{
    execute_buy_order::execute_buy_order, execute_sell_order::execute_sell_order,
    execute_transfer::execute_transfer, solve_quadratic_ask::solve_quadratic_ask,
    solve_quadratic_bid::solve_quadratic_bid, submit_buy_order::submit_buy_order,
    submit_sell_order::submit_sell_order, update_market_data::update_market_data,
};
use alloy_primitives::{Address, U128};
use common::vector::Vector;
use common_contracts::contracts::{
    clerk::{ClerkStorage, SCRATCH_1, SCRATCH_2, SCRATCH_3, SCRATCH_4},
    formulas::Order,
    keep::{Keep, Vault},
    keep_calls::KeepCalls,
};
use stylus_sdk::{abi::Bytes, prelude::*};
use vector_macros::amount_vec;

#[storage]
#[entrypoint]
pub struct Factor;

fn _init_solve_quadratic_bid(storage: &mut Keep, clerk_storage: &mut ClerkStorage) -> U128 {
    // Q_buy = (sqrt(P^2 + 4 * S * C_buy) - P) / 2 * S
    let solve_quadratic_id = {
        let mut id = storage.solve_quadratic_bid_id.get();
        if id.is_zero() {
            id = clerk_storage.next_vector();
            let code = solve_quadratic_bid();
            clerk_storage.store_bytes(id.to(), code);
            storage.solve_quadratic_bid_id.set(id);
            id
        } else {
            id
        }
    };
    solve_quadratic_id
}

fn _init_solve_quadratic_ask(storage: &mut Keep, clerk_storage: &mut ClerkStorage) -> U128 {
    // Q_sell = (P - sqrt(P^2 - 4 * S * C_sell)) / 2 * S
    let solve_quadratic_id = {
        let mut id = storage.solve_quadratic_ask_id.get();
        if id.is_zero() {
            id = clerk_storage.next_vector();
            let code = solve_quadratic_ask();
            clerk_storage.store_bytes(id.to(), code);
            storage.solve_quadratic_ask_id.set(id);
            id
        } else {
            id
        }
    };
    solve_quadratic_id
}

fn _init_trader_bid(
    vault: &mut Vault,
    clerk_storage: &mut ClerkStorage,
    trader_address: Address,
) -> U128 {
    let mut set_bid_id = vault.traders_bids.setter(trader_address);

    let bid_id = set_bid_id.get();
    if !bid_id.is_zero() {
        return bid_id;
    }

    let bid_id = clerk_storage.next_vector();
    set_bid_id.set(bid_id);

    clerk_storage.store_vector(bid_id.to(), amount_vec![0, 0, 0]);

    if vault.traders_asks.get(trader_address).is_zero() {
        vault.traders.push(trader_address);
    }

    bid_id
}

fn _init_trader_ask(
    vault: &mut Vault,
    clerk_storage: &mut ClerkStorage,
    trader_address: Address,
) -> U128 {
    let mut set_ask_id = vault.traders_asks.setter(trader_address);

    let ask_id = set_ask_id.get();
    if !ask_id.is_zero() {
        return ask_id;
    }

    let ask_id = clerk_storage.next_vector();
    set_ask_id.set(ask_id);

    clerk_storage.store_vector(ask_id.to(), amount_vec![0, 0, 0]);

    if vault.traders_bids.get(trader_address).is_zero() {
        vault.traders.push(trader_address);
    }

    ask_id
}

fn _get_vendor_quote_id(vault: &mut Vault, vendor_id: U128) -> Result<U128, Vec<u8>> {
    let quote_id = vault.vendor_quotes.get(vendor_id);
    if quote_id.is_zero() {
        Err(b"Quote not set")?;
    }

    return Ok(quote_id);
}

fn _init_vendor_bid(vault: &mut Vault, clerk_storage: &mut ClerkStorage, vendor_id: U128) -> U128 {
    let mut set_bid_id = vault.vendors_bids.setter(vendor_id);

    let bid_id = set_bid_id.get();
    if !bid_id.is_zero() {
        return bid_id;
    }

    let bid_id = clerk_storage.next_vector();
    set_bid_id.set(bid_id);

    clerk_storage.store_vector(bid_id.to(), amount_vec![0, 0, 0]);

    if vault.vendor_quotes.get(vendor_id).is_zero() && vault.vendors_asks.get(vendor_id).is_zero() {
        vault.vendors.push(vendor_id);
    }

    bid_id
}

fn _init_vendor_ask(vault: &mut Vault, clerk_storage: &mut ClerkStorage, vendor_id: U128) -> U128 {
    let mut set_ask_id = vault.vendors_asks.setter(vendor_id);

    let ask_id = set_ask_id.get();
    if !ask_id.is_zero() {
        return ask_id;
    }

    let ask_id = clerk_storage.next_vector();
    set_ask_id.set(ask_id);

    clerk_storage.store_vector(ask_id.to(), amount_vec![0, 0, 0]);

    if vault.vendor_quotes.get(vendor_id).is_zero() && vault.vendors_bids.get(vendor_id).is_zero() {
        vault.vendors.push(vendor_id);
    }

    ask_id
}

fn _init_total_bid(vault: &mut Vault, clerk_storage: &mut ClerkStorage) -> U128 {
    let bid_id = vault.total_bid.get();
    if !bid_id.is_zero() {
        return bid_id;
    }

    let bid_id = clerk_storage.next_vector();
    vault.total_bid.set(bid_id);

    clerk_storage.store_vector(bid_id.to(), amount_vec![0, 0, 0]);

    bid_id
}

fn _init_total_ask(vault: &mut Vault, clerk_storage: &mut ClerkStorage) -> U128 {
    let ask_id = vault.total_ask.get();
    if !ask_id.is_zero() {
        return ask_id;
    }

    let ask_id = clerk_storage.next_vector();
    vault.total_ask.set(ask_id);

    clerk_storage.store_vector(ask_id.to(), amount_vec![0, 0, 0]);

    ask_id
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
        asset_names: Bytes,
        asset_liquidity: Bytes,
        asset_prices: Bytes,
        asset_slopes: Bytes,
    ) -> Result<(), Vec<u8>> {
        let mut storage = Keep::storage();
        storage.check_version()?;

        let account = storage.accounts.setter(vendor_id);
        account.only_owner(self.attendee())?;

        let asset_names_id = SCRATCH_1;
        let asset_liquidity_id = SCRATCH_2;
        let asset_prices_id = SCRATCH_3;
        let asset_slopes_id = SCRATCH_4;

        let mut clerk_storage = ClerkStorage::storage();
        clerk_storage.store_bytes(asset_names_id, asset_names);
        clerk_storage.store_bytes(asset_liquidity_id, asset_liquidity);
        clerk_storage.store_bytes(asset_prices_id, asset_prices);
        clerk_storage.store_bytes(asset_slopes_id, asset_slopes);

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

        let clerk = storage.clerk.get();
        let num_registry = 16;
        self.update_records(clerk, update, num_registry)?;
        Ok(())
    }

    pub fn process_trader_buy_order(
        &mut self,
        vendor_id: U128,
        index_id: U128,
        trader_address: Address,
        max_order_size: u128,
        asset_contribution_fractions: Bytes,
    ) -> Result<(Bytes, Bytes, Bytes), Vec<u8>> {
        self.execute_buy_order(
            vendor_id,
            index_id,
            trader_address,
            0,
            0,
            max_order_size,
            asset_contribution_fractions,
        )
    }

    pub fn process_trader_sell_order(
        &mut self,
        vendor_id: U128,
        index_id: U128,
        trader_address: Address,
        max_order_size: u128,
        asset_contribution_fractions: Bytes,
    ) -> Result<(Bytes, Bytes, Bytes), Vec<u8>> {
        self.execute_sell_order(
            vendor_id,
            index_id,
            trader_address,
            0,
            0,
            max_order_size,
            asset_contribution_fractions,
        )
    }

    pub fn submit_buy_order(
        &mut self,
        vendor_id: U128,
        index_id: U128,
        trader_address: Address,
        collateral_added: u128,
        collateral_removed: u128,
    ) -> Result<(), Vec<u8>> {
        let mut storage = Keep::storage();
        storage.check_version()?;

        let mut clerk_storage = ClerkStorage::storage();

        let mut vault = storage.vaults.setter(index_id);

        // Allocate new Index order or get existing one
        let index_order_id = _init_trader_bid(&mut vault, &mut clerk_storage, trader_address);
        let vendor_order_id = _init_vendor_bid(&mut vault, &mut clerk_storage, vendor_id);
        let total_order_id = _init_total_bid(&mut vault, &mut clerk_storage);

        let update = submit_buy_order(
            index_order_id.to(),
            vendor_order_id.to(),
            total_order_id.to(),
            collateral_added,
            collateral_removed,
        );

        let clerk = storage.clerk.get();
        let num_registry = 9;
        self.update_records(clerk, update, num_registry)?;

        Ok(())
    }

    pub fn submit_sell_order(
        &mut self,
        vendor_id: U128,
        index_id: U128,
        trader_address: Address,
        collateral_added: u128,
        collateral_removed: u128,
    ) -> Result<(), Vec<u8>> {
        let mut storage = Keep::storage();
        storage.check_version()?;

        let mut clerk_storage = ClerkStorage::storage();

        let mut vault = storage.vaults.setter(index_id);

        // Allocate new Index order or get existing one
        let index_order_id = _init_trader_ask(&mut vault, &mut clerk_storage, trader_address);
        let vendor_order_id = _init_vendor_ask(&mut vault, &mut clerk_storage, vendor_id);
        let total_order_id = _init_total_ask(&mut vault, &mut clerk_storage);

        let update = submit_sell_order(
            index_order_id.to(),
            vendor_order_id.to(),
            total_order_id.to(),
            collateral_added,
            collateral_removed,
        );

        let clerk = storage.clerk.get();
        let num_registry = 9;
        self.update_records(clerk, update, num_registry)?;

        Ok(())
    }

    /// Submit BUY Index order
    ///
    /// Add collateral amount to user's order, and match for immediate execution.
    ///
    pub fn execute_buy_order(
        &mut self,
        vendor_id: U128,
        index_id: U128,
        trader_address: Address,
        collateral_added: u128,
        collateral_removed: u128,
        max_order_size: u128,
        asset_contribution_fractions: Bytes,
    ) -> Result<(Bytes, Bytes, Bytes), Vec<u8>> {
        let mut storage = Keep::storage();
        storage.check_version()?;

        let mut clerk_storage = ClerkStorage::storage();

        // Allocate Quadratic Solver
        let solve_quadratic_id = _init_solve_quadratic_bid(&mut storage, &mut clerk_storage);

        let mut vault = storage.vaults.setter(index_id);

        let vendor_quote_id = _get_vendor_quote_id(&mut vault, vendor_id)?;

        // Allocate new Index order or get existing one
        let index_order_id = _init_trader_bid(&mut vault, &mut clerk_storage, trader_address);
        let vendor_order_id = _init_vendor_bid(&mut vault, &mut clerk_storage, vendor_id);
        let total_order_id = _init_total_bid(&mut vault, &mut clerk_storage);

        let account = storage.accounts.get(vendor_id);

        let asset_contribution_fractions_id = SCRATCH_1;

        clerk_storage.store_bytes(
            asset_contribution_fractions_id,
            asset_contribution_fractions,
        );

        let executed_asset_quantities_id = SCRATCH_2;
        let executed_index_quantities_id = SCRATCH_3;

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

        let clerk = storage.clerk.get();
        let num_registry = 23;
        self.update_records(clerk, update, num_registry)?;

        let index_order = clerk_storage
            .fetch_bytes(index_order_id)
            .ok_or_else(|| b"Index order not set")?;

        let executed_asset_quantities = clerk_storage
            .fetch_bytes(executed_asset_quantities_id)
            .ok_or_else(|| b"Executed asset quantities not set")?;

        let executed_index_quantities = clerk_storage
            .fetch_bytes(executed_index_quantities_id)
            .ok_or_else(|| b"Executed index quantities not set")?;

        Ok((
            index_order.into(),
            executed_index_quantities.into(),
            executed_asset_quantities.into(),
        ))
    }

    pub fn execute_sell_order(
        &mut self,
        vendor_id: U128,
        index_id: U128,
        trader_address: Address,
        collateral_added: u128,
        collateral_removed: u128,
        max_order_size: u128,
        asset_contribution_fractions: Bytes,
    ) -> Result<(Bytes, Bytes, Bytes), Vec<u8>> {
        let mut storage = Keep::storage();
        storage.check_version()?;

        let mut clerk_storage = ClerkStorage::storage();

        // Allocate Quadratic Solver
        let solve_quadratic_id = _init_solve_quadratic_ask(&mut storage, &mut clerk_storage);

        let mut vault = storage.vaults.setter(index_id);

        let vendor_quote_id = _get_vendor_quote_id(&mut vault, vendor_id)?;

        // Allocate new Index order or get existing one
        let sender_bid_id = _init_trader_bid(&mut vault, &mut clerk_storage, trader_address);
        let sender_ask_id = _init_trader_ask(&mut vault, &mut clerk_storage, trader_address);
        let vendor_order_id = _init_vendor_ask(&mut vault, &mut clerk_storage, vendor_id);
        let total_order_id = _init_total_ask(&mut vault, &mut clerk_storage);
        
        let sender_bid_bytes = clerk_storage
            .fetch_bytes(sender_bid_id)
            .ok_or_else(|| b"Sender Bid not set (sell)")?;

        let sender_ask_bytes = clerk_storage
            .fetch_bytes(sender_ask_id)
            .ok_or_else(|| b"Sender Ask not set (sell)")?;

        let order = Order::try_from_vec_pair(sender_bid_bytes, sender_ask_bytes)?;

        if order.tell_available()?.to_u128_raw() < collateral_added {
            Err(b"Insufficient amount of Index token (sell)")?;
        }

        let account = storage.accounts.get(vendor_id);

        let asset_contribution_fractions_id = SCRATCH_1;

        clerk_storage.store_bytes(
            asset_contribution_fractions_id,
            asset_contribution_fractions,
        );

        let executed_asset_quantities_id = SCRATCH_2;
        let executed_index_quantities_id = SCRATCH_3;

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
            sender_ask_id.to(), // single trader orders aggregated per vault (we don't store individual orders)
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

        let clerk = storage.clerk.get();
        let num_registry = 22;
        self.update_records(clerk, update, num_registry)?;

        let index_order = clerk_storage
            .fetch_bytes(sender_ask_id)
            .ok_or_else(|| b"Index order not set")?;

        let executed_asset_quantities = clerk_storage
            .fetch_bytes(executed_asset_quantities_id)
            .ok_or_else(|| b"Executed asset quantities not set")?;

        let executed_index_quantities = clerk_storage
            .fetch_bytes(executed_index_quantities_id)
            .ok_or_else(|| b"Executed index quantities not set")?;

        Ok((
            index_order.into(),
            executed_index_quantities.into(),
            executed_asset_quantities.into(),
        ))
    }

    // pub fn submit_rebalance_order(
    //     &mut self,
    //     vendor_id: U128,
    //     new_assets: Vec<u8>,
    //     new_weigthts: Vec<u8>,
    // ) -> Result<(), Vec<u8>> {
    //     //
    //     // This needs to:
    //     //  - compute rebalance_weights_long = max(0, weights - new_weights) -- assets long in inventory (sell them)
    //     //  - compute rebalance_weights_short = max(0, new_weights - weights) -- assets short in inventory (buy more)
    //     //
    //     Err(b"Not implemented yet".into())
    // }

    pub fn execute_transfer(
        &mut self,
        index_id: U128,
        sender: Address,
        receiver: Address,
        amount: u128,
    ) -> Result<(), Vec<u8>> {
        let mut storage = Keep::storage();
        storage.check_version()?;

        let mut vault = storage.vaults.setter(index_id);
        let mut clerk_storage = ClerkStorage::storage();

        // Transfers are initiated by Vaults on behalf of users and not
        // by users themselves. This way it is more efficient.
        // if vault.gate_to_vault.get() != self.attendee() {
        // Err(b"Incorrect Vault")?;
        // }

        // Note here we need both Bid & Ask for sender account, but only Bid for
        // receiver account. The receiver will obtain new ITP in Minted column
        // together with split cost basis in Spent column of their Bid vector.
        // We will take Minted colunm from senders Bid vector, and we must subtract
        // the ITP that sender has currently locked for redeeming by taking Remain
        // column together with Spent column reflecting ITP they redeemed (burned)
        // of their Ask vector. Transfer performs rebalancing by splitting cost basis
        // together with moving minted ITP amount.
        let sender_bid_id = _init_trader_bid(&mut vault, &mut clerk_storage, sender);
        let sender_ask_id = _init_trader_ask(&mut vault, &mut clerk_storage, sender);
        let receiver_bid_id = _init_trader_bid(&mut vault, &mut clerk_storage, receiver);

        // Optional check: We don't need to check balance here as VIL program
        // will fail if balance is insufficient, however we want to produce
        // friendly error message insted of VIL program error.
        let sender_bid_bytes = clerk_storage
            .fetch_bytes(sender_bid_id)
            .ok_or_else(|| b"Sender Bid not set")?;

        let sender_ask_bytes = clerk_storage
            .fetch_bytes(sender_ask_id)
            .ok_or_else(|| b"Sender Ask not set")?;

        let order = Order::try_from_vec_pair(sender_bid_bytes, sender_ask_bytes)?;

        if order.tell_available()?.to_u128_raw() < amount {
            Err(b"Insufficient amount of Index token")?;
        }

        // Transfer Assets & Liabilities from account A to account B
        //
        // Note: We perform meticulous rebalancing here where side A
        // gets Minted amount deducted together with Spent, so that
        // we split cost basis between account A and account B.
        //
        let update = execute_transfer(
            sender_bid_id.to(),
            sender_ask_id.to(),
            receiver_bid_id.to(),
            amount,
        );

        let clerk = storage.clerk.get();
        let num_registry = 6;
        self.update_records(clerk, update, num_registry)?;

        Ok(())
    }
}
