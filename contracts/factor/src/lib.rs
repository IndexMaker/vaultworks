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
    submit_sell_order::submit_sell_order,
};
use alloy_primitives::{Address, U128};
use common::{amount::Amount, vector::Vector};
use common_contracts::contracts::{
    clerk::{ClerkStorage, SCRATCH_1, SCRATCH_2},
    formulas::{Order, ORDER_REMAIN_OFFSET},
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
            clerk_storage.store_bytes(id.to(), code.unwrap());
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
            clerk_storage.store_bytes(id.to(), code.unwrap());
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

impl Factor {
    fn _transfer_buy_to_operator(
        &mut self,
        vault: &mut Vault,
        clerk_storage: &mut ClerkStorage,
        operator_address: Address,
        index_order_id: U128,
    ) -> Result<(), Vec<u8>> {
        let operator_order_id = _init_trader_bid(vault, clerk_storage, operator_address);

        let mut operator_order = clerk_storage
            .fetch_vector(operator_order_id)
            .ok_or_else(|| b"Index order not set")?;

        let mut trader_order = clerk_storage
            .fetch_vector(index_order_id)
            .ok_or_else(|| b"Index order not set")?;

        // Carry over of Collateral Remain from Trader to Operator
        // is a straight-forward process. We zero C/Remain for Trader
        // and add to Operator's C/Remain.
        let collateral_remain = trader_order.data[ORDER_REMAIN_OFFSET];
        let mut operator_collateral = operator_order.data[ORDER_REMAIN_OFFSET];

        operator_collateral = operator_collateral
            .checked_add(collateral_remain)
            .ok_or_else(|| b"MathOverflow")?;

        operator_order.data[ORDER_REMAIN_OFFSET] = operator_collateral;
        trader_order.data[ORDER_REMAIN_OFFSET] = Amount::ZERO;

        clerk_storage.store_vector(operator_order_id, operator_order);
        clerk_storage.store_vector(index_order_id, trader_order);

        Ok(())
    }

    fn _transfer_sell_to_operator(
        &mut self,
        vault: &mut Vault,
        clerk_storage: &mut ClerkStorage,
        clerk: Address,
        operator_address: Address,
        sender_bid_id: U128,
        sender_ask_id: U128,
    ) -> Result<(), Vec<u8>> {
        let mut trader_ask = clerk_storage
            .fetch_vector(sender_ask_id)
            .ok_or_else(|| b"Index order not set")?;

        // This is pretty intricate:
        // - First we need to zero ITP Locked for Trader,
        //   buy we must carry it over to Operator later on (...)
        let itp_locked = trader_ask.data[ORDER_REMAIN_OFFSET];
        trader_ask.data[ORDER_REMAIN_OFFSET] = Amount::ZERO;

        clerk_storage.store_vector(sender_ask_id, trader_ask);

        let operator_bid_id = _init_trader_bid(vault, clerk_storage, operator_address);
        let operator_ask_id = _init_trader_ask(vault, clerk_storage, operator_address);

        // - Before we carry over ITP Locked to Operator, we must first
        //   transfer that amount of ITP from Trader to Operator, and then (...)
        let update = execute_transfer(
            sender_bid_id.to(),
            sender_ask_id.to(),
            operator_bid_id.to(),
            itp_locked.to_u128_raw(),
        );

        let num_registry = 6;
        self.update_records(clerk, update?, num_registry)?;

        // - Once we have transferred ITP from Trader to Operator, we can now
        //   carry over ITP Locked to Operator
        //
        // NOTE: While this process might seem complex, it is necessary to zero
        // ITP Locked for Trader before making transfer, as otherwise transfer
        // would fail. Also it would be incorrect to carry over ITP Locked before
        // transfer, as that would result in negative balance of the Operator.
        let mut operator_ask_order = clerk_storage
            .fetch_vector(operator_ask_id)
            .ok_or_else(|| b"Index order not set")?;

        let operator_itp_locked = operator_ask_order.data[ORDER_REMAIN_OFFSET];
        operator_ask_order.data[ORDER_REMAIN_OFFSET] = operator_itp_locked
            .checked_add(itp_locked)
            .ok_or_else(|| b"MathOverflow")?;

        clerk_storage.store_vector(operator_ask_id, operator_ask_order);

        Ok(())
    }

    fn _execute_buy_order(
        &mut self,
        vendor_id: U128,
        index_id: U128,
        trader_address: Address,
        operator_address: Address,
        collateral_added: u128,
        collateral_removed: u128,
        max_order_size: u128,
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), Vec<u8>> {
        let mut storage = Keep::storage();
        storage.check_version()?;

        let mut clerk_storage = ClerkStorage::storage();

        // Allocate Quadratic Solver
        let solve_quadratic_id = _init_solve_quadratic_bid(&mut storage, &mut clerk_storage);

        let mut vault = storage.vaults.setter(index_id);
        vault.only_tradeable()?;

        let vendor_quote_id = _get_vendor_quote_id(&mut vault, vendor_id)?;

        // Allocate new Index order or get existing one
        let index_order_id = _init_trader_bid(&mut vault, &mut clerk_storage, trader_address);
        let vendor_order_id = _init_vendor_bid(&mut vault, &mut clerk_storage, vendor_id);
        let total_order_id = _init_total_bid(&mut vault, &mut clerk_storage);

        let account = storage.accounts.get(vendor_id);

        let executed_asset_quantities_id = SCRATCH_1;
        let executed_index_quantities_id = SCRATCH_2;

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
            solve_quadratic_id.to(),
        );

        let clerk = storage.clerk.get();
        let num_registry = 23;
        self.update_records(clerk, update?, num_registry)?;

        if operator_address != trader_address {
            self._transfer_buy_to_operator(
                &mut vault,
                &mut clerk_storage,
                operator_address,
                index_order_id,
            )?;
        }

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
            index_order,
            executed_index_quantities,
            executed_asset_quantities,
        ))
    }

    fn _execute_sell_order(
        &mut self,
        vendor_id: U128,
        index_id: U128,
        trader_address: Address,
        operator_address: Address,
        collateral_added: u128,
        collateral_removed: u128,
        max_order_size: u128,
    ) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), Vec<u8>> {
        let mut storage = Keep::storage();
        storage.check_version()?;

        let mut clerk_storage = ClerkStorage::storage();

        // Allocate Quadratic Solver
        let solve_quadratic_id = _init_solve_quadratic_ask(&mut storage, &mut clerk_storage);

        let mut vault = storage.vaults.setter(index_id);
        vault.only_tradeable()?;

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

        let executed_asset_quantities_id = SCRATCH_1;
        let executed_index_quantities_id = SCRATCH_2;

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
            solve_quadratic_id.to(),
        );

        let clerk = storage.clerk.get();
        let num_registry = 22;
        self.update_records(clerk, update?, num_registry)?;

        if operator_address != trader_address {
            self._transfer_sell_to_operator(
                &mut vault,
                &mut clerk_storage,
                clerk,
                operator_address,
                sender_bid_id,
                sender_ask_id,
            )?;
        }

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
            index_order,
            executed_index_quantities,
            executed_asset_quantities,
        ))
    }
}

#[public]
impl Factor {
    pub fn submit_buy_order(
        &mut self,
        vendor_id: U128,
        index_id: U128,
        trader_address: Address,
        collateral_added: u128,
        collateral_removed: u128,
    ) -> Result<(), Vec<u8>> {
        if vendor_id.is_zero() {
            Err(b"Vendor ID cannot be zero")?;
        }
        if index_id.is_zero() {
            Err(b"Index ID cannot be zero")?;
        }
        if trader_address.is_zero() {
            Err(b"Trader Address cannot be zero")?;
        }

        let mut storage = Keep::storage();
        storage.check_version()?;

        let mut clerk_storage = ClerkStorage::storage();

        let mut vault = storage.vaults.setter(index_id);
        vault.only_tradeable()?;

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
        self.update_records(clerk, update?, num_registry)?;

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
        if vendor_id.is_zero() {
            Err(b"Vendor ID cannot be zero")?;
        }
        if index_id.is_zero() {
            Err(b"Index ID cannot be zero")?;
        }
        if trader_address.is_zero() {
            Err(b"Trader Address cannot be zero")?;
        }

        let mut storage = Keep::storage();
        storage.check_version()?;

        let mut clerk_storage = ClerkStorage::storage();

        let mut vault = storage.vaults.setter(index_id);
        vault.only_tradeable()?;

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
        self.update_records(clerk, update?, num_registry)?;

        Ok(())
    }

    pub fn process_pending_buy_order(
        &mut self,
        vendor_id: U128,
        index_id: U128,
        trader_address: Address,
        max_order_size: u128,
    ) -> Result<Vec<Bytes>, Vec<u8>> {
        if vendor_id.is_zero() {
            Err(b"Vendor ID cannot be zero")?;
        }
        if index_id.is_zero() {
            Err(b"Index ID cannot be zero")?;
        }
        if trader_address.is_zero() {
            Err(b"Trader Address cannot be zero")?;
        }
        if max_order_size == 0 {
            Err(b"MaxOrderSize cannot be zero")?;
        }

        let (index_order, executed_index_quantities, executed_asset_quantities) = self
            ._execute_buy_order(
                vendor_id,
                index_id,
                trader_address,
                trader_address,
                0,
                0,
                max_order_size,
            )?;

        Ok(vec![
            index_order.into(),
            executed_index_quantities.into(),
            executed_asset_quantities.into(),
        ])
    }

    pub fn process_pending_sell_order(
        &mut self,
        vendor_id: U128,
        index_id: U128,
        trader_address: Address,
        max_order_size: u128,
    ) -> Result<Vec<Bytes>, Vec<u8>> {
        if vendor_id.is_zero() {
            Err(b"Vendor ID cannot be zero")?;
        }
        if index_id.is_zero() {
            Err(b"Index ID cannot be zero")?;
        }
        if trader_address.is_zero() {
            Err(b"Trader Address cannot be zero")?;
        }
        if max_order_size == 0 {
            Err(b"MaxOrderSize cannot be zero")?;
        }

        let (index_order, executed_index_quantities, executed_asset_quantities) = self
            ._execute_sell_order(
                vendor_id,
                index_id,
                trader_address,
                trader_address,
                0,
                0,
                max_order_size,
            )?;

        Ok(vec![
            index_order.into(),
            executed_index_quantities.into(),
            executed_asset_quantities.into(),
        ])
    }

    /// Execute BUY Index order
    ///
    /// Add collateral amount to user's order, and match for immediate execution.
    ///
    /// Any remaining collateral is transferred to operator for further execution.
    ///
    pub fn execute_buy_order(
        &mut self,
        vendor_id: U128,
        index_id: U128,
        trader_address: Address,
        operator_address: Address,
        collateral_amount: u128,
        max_order_size: u128,
    ) -> Result<Vec<Bytes>, Vec<u8>> {
        if vendor_id.is_zero() {
            Err(b"Vendor ID cannot be zero")?;
        }
        if index_id.is_zero() {
            Err(b"Index ID cannot be zero")?;
        }
        if trader_address.is_zero() {
            Err(b"Trader Address cannot be zero")?;
        }
        if operator_address.is_zero() {
            Err(b"Trader Address and Operator Address must differ")?;
        }
        if max_order_size == 0 {
            Err(b"MaxOrderSize cannot be zero")?;
        }

        let (index_order, executed_index_quantities, executed_asset_quantities) = self
            ._execute_buy_order(
                vendor_id,
                index_id,
                trader_address,
                operator_address,
                collateral_amount,
                0,
                max_order_size,
            )?;

        Ok(vec![
            index_order.into(),
            executed_index_quantities.into(),
            executed_asset_quantities.into(),
        ])
    }

    /// Execute SELL Index order
    ///
    /// Add ITP amount to user's order, and match for immediate execution.
    ///
    /// Any remaining ITP is transferred to operator for further execution.
    ///
    pub fn execute_sell_order(
        &mut self,
        vendor_id: U128,
        index_id: U128,
        trader_address: Address,
        operator_address: Address,
        itp_amount: u128,
        max_order_size: u128,
    ) -> Result<Vec<Bytes>, Vec<u8>> {
        if vendor_id.is_zero() {
            Err(b"Vendor ID cannot be zero")?;
        }
        if index_id.is_zero() {
            Err(b"Index ID cannot be zero")?;
        }
        if trader_address.is_zero() {
            Err(b"Trader Address cannot be zero")?;
        }
        if operator_address.is_zero() {
            Err(b"Trader Address and Operator Address must differ")?;
        }
        if max_order_size == 0 {
            Err(b"MaxOrderSize cannot be zero")?;
        }

        let (index_order, executed_index_quantities, executed_asset_quantities) = self
            ._execute_sell_order(
                vendor_id,
                index_id,
                trader_address,
                operator_address,
                itp_amount,
                0,
                max_order_size,
            )?;

        Ok(vec![
            index_order.into(),
            executed_index_quantities.into(),
            executed_asset_quantities.into(),
        ])
    }

    /// Execute Transfer from Sender to Receiver
    ///
    /// This transfers both ITP and proportionalcollateral cost.
    ///
    pub fn execute_transfer(
        &mut self,
        index_id: U128,
        sender: Address,
        receiver: Address,
        amount: u128,
    ) -> Result<(), Vec<u8>> {
        if index_id.is_zero() {
            Err(b"Index ID cannot be zero")?;
        }
        if sender.is_zero() {
            Err(b"Sender cannot be zero")?;
        }
        if receiver.is_zero() {
            Err(b"Receiver cannot be zero")?;
        }
        if amount == 0 {
            return Ok(());
        }

        let mut storage = Keep::storage();
        storage.check_version()?;

        let mut vault = storage.vaults.setter(index_id);
        let mut clerk_storage = ClerkStorage::storage();
        vault.only_tradeable()?;

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
        self.update_records(clerk, update?, num_registry)?;

        Ok(())
    }
}
