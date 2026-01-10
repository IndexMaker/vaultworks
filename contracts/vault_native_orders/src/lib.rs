// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U128};
use alloy_sol_types::{sol, SolEvent};
use common::vector::Vector;
use common_contracts::{
    contracts::{
        calls::InnerCall,
        formulas::{Report, ORDER_REMAIN_OFFSET},
        keep_calls::KeepCalls,
        vault::VaultStorage,
        vault_native::VaultNativeStorage,
    },
    interfaces::{
        factor::IFactor,
        vault_native_orders::IVaultNativeOrders::{Acquisition, BuyOrder, Disposal, SellOrder},
    },
};
use stylus_sdk::{prelude::*, ArbResult};

sol! {
    interface IERC20 {
        function transferFrom(address from, address to, uint256 value) external returns (bool);
    }
}

#[storage]
#[entrypoint]
pub struct VaultNativeOrders;

#[public]
impl VaultNativeOrders {
    /// Places new BUY order request into the network.
    ///
    /// This takes the deposit into custody account, and fires an event, which
    /// will be picked by Keeper service to perform actual order processing.
    ///
    /// An option of an instant fill allows users to get their order executed
    /// immediately. However there are drawbacks of an instant fill:
    /// - higher gas cost as user must pay for quote update and order execution
    /// - execution prices will be off as vendor might not have supplied fresh market data
    /// - executed quantity will be capped at MaxOrderSize
    ///
    pub fn place_buy_order(
        &mut self,
        collateral_amount: U128,
        instant_fill: bool,
        keeper: Address,
        trader: Address,
    ) -> Result<(U128, U128, U128), Vec<u8>> {
        if collateral_amount.is_zero() {
            Err(b"Zero collateral amount")?;
        }
        let vault = VaultStorage::storage();
        let mut requests = VaultNativeStorage::storage();
        let sender = self.attendee();

        // Order can be placed by either trader or an operator elected by trader.
        // e.g. another smart-contract can act on behalf of trader.
        if sender != trader && !requests.is_operator(trader, sender) {
            Err(b"Unauthorised order placement")?;
        }

        // Transfer USDC collateral from user to dedicated custody
        let asset = requests.collateral_asset.get();
        self.external_call(
            asset,
            IERC20::transferFromCall {
                from: trader,
                to: requests.custody.get(),
                value: collateral_amount.to(),
            },
        )?;

        // Submit order and get instant fill if possible
        let (delivered, received, request_event) = if instant_fill {
            // We should use fresh prices
            requests.update_quote(&vault, self)?;

            let ret = self.external_call_ret(
                vault.gate_to_castle.get(),
                IFactor::executeBuyOrderCall {
                    vendor_id: requests.vendor_id.get().to(),
                    index_id: vault.index_id.get().to(),
                    trader_address: trader,
                    operator_address: keeper,
                    collateral_amount: collateral_amount.to(),
                    max_order_size: requests.max_order_size.get().to(),
                },
            )?;
            let report = Report::try_from_vec(ret._0[1].to_vec())?;
            (
                report.delivered().to_u128(),
                report.received().to_u128(),
                // We publish event with zero collateral added, as we already
                // updated our order in Clerk Chamber, and if fill was only partial,
                // then Keeper needs to continue filling.
                BuyOrder {
                    keeper,
                    index_id: vault.index_id.get().to(),
                    vendor_id: requests.vendor_id.get().to(),
                    collateral_amount: 0,
                    trader,
                },
            )
        } else {
            // Send pending order without executing it
            self.external_call(
                vault.gate_to_castle.get(),
                IFactor::submitBuyOrderCall {
                    vendor_id: requests.vendor_id.get().to(),
                    index_id: vault.index_id.get().to(),
                    trader_address: keeper,
                    collateral_added: collateral_amount.to(),
                    collateral_removed: 0,
                },
            )?;

            (
                U128::ZERO,
                U128::ZERO,
                // We publish event with original collateral amount, as we only
                // deposited collateral, but haven't executed anything yet. Keeper
                // service will call submitByOrder() on our behalf.
                BuyOrder {
                    keeper,
                    index_id: vault.index_id.get().to(),
                    vendor_id: requests.vendor_id.get().to(),
                    collateral_amount: collateral_amount.to(),
                    trader,
                },
            )
        };

        // Store operator's liability towards the trader
        let mut trader_orders = requests.trader_orders.setter(trader);
        let mut pending_bid = trader_orders.pending_bid.setter(keeper);

        let collateral_remain = collateral_amount
            .checked_sub(delivered)
            .ok_or_else(|| b"MathUnderflow")?;

        let pending_amount = pending_bid
            .get()
            .checked_add(collateral_remain)
            .ok_or_else(|| b"MathUnderflow")?;

        pending_bid.set(pending_amount);

        if !received.is_zero() {
            // Publish execution report if there was execution

            let exec_report = Acquisition {
                controller: trader,
                index_id: vault.index_id.get().to(),
                vendor_id: requests.vendor_id.get().to(),
                remain: collateral_remain.to(),
                spent: delivered.to(),
                itp_minted: received.to(),
            };

            self.vm().emit_log(&exec_report.encode_data(), 1);
        }

        // Send an event, and it will be picked up by Keeper service
        self.vm().emit_log(&request_event.encode_data(), 1);

        Ok((delivered, received, collateral_remain))
    }

    /// Places new SELL order request into the network.
    ///
    /// This only fires an event, which will be picked by Keeper service to
    /// perform actual order processing.
    ///
    /// Once Keeper service realizes the order, the on-chain state will reflect
    /// that in the Clerk Chamber, and an off-chain service needs to deposit
    /// gains to custody account so that user can claim them.
    ///
    /// An option of an instant fill allows users to get their order executed
    /// immediately. However there are drawbacks of an instant fill:
    /// - higher gas cost as user must pay for quote update and order execution
    /// - execution prices will be off as vendor might not have supplied fresh market data
    /// - executed quantity will be capped at MaxOrderSize
    ///
    pub fn place_sell_order(
        &mut self,
        itp_amount: U128,
        instant_fill: bool,
        keeper: Address,
        trader: Address,
    ) -> Result<(U128, U128, U128), Vec<u8>> {
        if itp_amount.is_zero() {
            Err(b"Zero ITP amount")?;
        }

        let vault = VaultStorage::storage();
        let mut requests = VaultNativeStorage::storage();
        let sender = self.attendee();

        // Order can be placed by either trader or an operator elected by trader.
        // e.g. another smart-contract can act on behalf of trader.
        if sender != trader && !requests.is_operator(trader, sender) {
            Err(b"Unauthorised order placement")?;
        }

        let (delivered, received, request_event) = if instant_fill {
            // We should use fresh prices
            requests.update_quote(&vault, self)?;

            // Submit order and get instant fill if possible
            let ret = self.external_call_ret(
                vault.gate_to_castle.get(),
                IFactor::executeSellOrderCall {
                    vendor_id: requests.vendor_id.get().to(),
                    index_id: vault.index_id.get().to(),
                    trader_address: trader,
                    operator_address: keeper,
                    itp_amount: itp_amount.to(),
                    max_order_size: requests.max_order_size.get().to(),
                },
            )?;
            let report = Report::try_from_vec(ret._0[1].to_vec())?;
            (
                report.delivered().to_u128(),
                report.received().to_u128(),
                // We publish event with zero ITP added, as we already updated our
                // order in Clerk Chamber, and if fill was only partial, then Keeper
                // needs to continue filling.
                SellOrder {
                    keeper,
                    index_id: vault.index_id.get().to(),
                    vendor_id: requests.vendor_id.get().to(),
                    itp_amount: 0,
                    trader,
                },
            )
        } else {
            // Send pending order without executing it
            self.external_call(
                vault.gate_to_castle.get(),
                IFactor::submitSellOrderCall {
                    vendor_id: requests.vendor_id.get().to(),
                    index_id: vault.index_id.get().to(),
                    trader_address: keeper,
                    collateral_added: itp_amount.to(),
                    collateral_removed: 0,
                },
            )?;

            (
                U128::ZERO,
                U128::ZERO,
                // We publish event with original ITP amount, as we haven't executed
                // anything yet. Keeper service will call submitByOrder() on our
                // behalf.
                SellOrder {
                    keeper,
                    index_id: vault.index_id.get().to(),
                    vendor_id: requests.vendor_id.get().to(),
                    itp_amount: itp_amount.to(),
                    trader,
                },
            )
        };

        // Store operator's liability towards the trader
        let mut trader_orders = requests.trader_orders.setter(trader);
        let mut pending_ask = trader_orders.pending_ask.setter(keeper);

        let itp_remain = itp_amount
            .checked_sub(delivered)
            .ok_or_else(|| b"MathUnderflow")?;

        let pending_amount = pending_ask
            .get()
            .checked_add(itp_remain)
            .ok_or_else(|| b"MathUnderflow")?;

        pending_ask.set(pending_amount);

        if !received.is_zero() {
            // Publish execution report if there was execution

            let exec_report = Disposal {
                controller: trader,
                index_id: vault.index_id.get().to(),
                vendor_id: requests.vendor_id.get().to(),
                itp_remain: itp_remain.to(),
                itp_burned: delivered.to(),
                gains: received.to(),
            };

            self.vm().emit_log(&exec_report.encode_data(), 1);
        }

        // Send an event, and it will be picked up by Keeper service.
        self.vm().emit_log(&request_event.encode_data(), 1);

        Ok((delivered, received, itp_remain))
    }

    /// Keeper can push forward pending orders
    pub fn process_pending_buy_order(
        &mut self,
        keeper: Address,
    ) -> Result<(U128, U128, U128), Vec<u8>> {
        let vault = VaultStorage::storage();
        let mut requests = VaultNativeStorage::storage();
        let sender = self.attendee();

        // Pending orders can be processed by either keeper or an operator
        // elected by keeper.
        if sender != keeper && !requests.is_operator(keeper, sender) {
            Err(b"Unauthorised order processing")?;
        }

        let ret = self.external_call_ret(
            vault.gate_to_castle.get(),
            IFactor::processPendingBuyOrderCall {
                vendor_id: requests.vendor_id.get().to(),
                index_id: vault.index_id.get().to(),
                trader_address: keeper,
                max_order_size: requests.max_order_size.get().to(),
            },
        )?;

        let order = Vector::from_vec(ret._0[0].to_vec());
        let pending_amount = order.data[ORDER_REMAIN_OFFSET].to_u128();

        let report = Report::try_from_vec(ret._0[1].to_vec())?;
        let delivered = report.delivered().to_u128();
        let received = report.received().to_u128();

        let mut operator_order = requests.opearator_order.setter(keeper);

        let delivered_amount = operator_order
            .bid_delivered
            .get()
            .checked_add(delivered)
            .ok_or_else(|| b"MathOverflow")?;

        operator_order.bid_delivered.set(delivered_amount);

        let received_amount = operator_order
            .ask_received
            .get()
            .checked_add(received)
            .ok_or_else(|| b"MathOverflow")?;

        operator_order.ask_received.set(received_amount);
        
        if !received.is_zero() {
            // Publish execution report if there was execution

            let exec_report = Acquisition {
                controller: keeper,
                index_id: vault.index_id.get().to(),
                vendor_id: requests.vendor_id.get().to(),
                remain: pending_amount.to(),
                spent: delivered.to(),
                itp_minted: received.to(),
            };

            self.vm().emit_log(&exec_report.encode_data(), 1);
        }

        Ok((delivered, received, pending_amount))
    }

    /// Keeper can push forward pending orders
    pub fn process_pending_sell_order(
        &mut self,
        keeper: Address,
    ) -> Result<(U128, U128, U128), Vec<u8>> {
        let vault = VaultStorage::storage();
        let mut requests = VaultNativeStorage::storage();
        let sender = self.attendee();

        // Pending orders can be processed by either keeper or an operator
        // elected by keeper.
        if sender != keeper && !requests.is_operator(keeper, sender) {
            Err(b"Unauthorised order processing")?;
        }

        let ret = self.external_call_ret(
            vault.gate_to_castle.get(),
            IFactor::processPendingBuyOrderCall {
                vendor_id: requests.vendor_id.get().to(),
                index_id: vault.index_id.get().to(),
                trader_address: keeper,
                max_order_size: requests.max_order_size.get().to(),
            },
        )?;

        let order = Vector::from_vec(ret._0[0].to_vec());
        let pending_amount = order.data[ORDER_REMAIN_OFFSET].to_u128();

        let report = Report::try_from_vec(ret._0[1].to_vec())?;
        let delivered = report.delivered().to_u128();
        let received = report.received().to_u128();

        let mut operator_order = requests.opearator_order.setter(keeper);

        let delivered_amount = operator_order
            .ask_delivered
            .get()
            .checked_add(delivered)
            .ok_or_else(|| b"MathOverflow")?;

        operator_order.ask_delivered.set(delivered_amount);

        let received_amount = operator_order
            .ask_received
            .get()
            .checked_add(received)
            .ok_or_else(|| b"MathOverflow")?;

        operator_order.ask_received.set(received_amount);
        
        if !received.is_zero() {
            // Publish execution report if there was execution

            let exec_report = Disposal {
                controller: keeper,
                index_id: vault.index_id.get().to(),
                vendor_id: requests.vendor_id.get().to(),
                itp_remain: pending_amount.to(),
                itp_burned: delivered.to(),
                gains: received.to(),
            };

            self.vm().emit_log(&exec_report.encode_data(), 1);
        }

        Ok((delivered, received, pending_amount))
    }

    #[payable]
    #[fallback]
    fn fallback(&mut self, calldata: &[u8]) -> ArbResult {
        let requests = {
            let requests = VaultNativeStorage::storage();
            let implementation = requests.claims_implementation.get();
            if implementation.is_zero() {
                Err(b"No claims implementation")?;
            }
            implementation
        };

        unsafe {
            let result = self.vm().delegate_call(&self, requests, calldata)?;
            Ok(result)
        }
    }
}
