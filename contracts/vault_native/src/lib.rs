// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U128};
use alloy_sol_types::{sol, SolEvent};
use common::{amount::Amount, log_msg};
use common_contracts::{
    contracts::{
        calls::InnerCall, keep_calls::KeepCalls, vault::VaultStorage,
        vault_native::VaultNativeStorage,
    },
    interfaces::{
        factor::IFactor,
        vault_native::IVaultNative::{BuyOrder, SellOrder},
    },
};
use stylus_sdk::prelude::*;

sol! {
    interface IERC20 {
        function transferFrom(address from, address to, uint256 value) external returns (bool);
    }
}

#[storage]
#[entrypoint]
pub struct VaultNative;

#[public]
impl VaultNative {
    pub fn configure_requests(
        &mut self,
        vendor_id: U128,
        custody: Address,
        asset: Address,
        max_order_size: U128,
    ) -> Result<(), Vec<u8>> {
        let vault = VaultStorage::storage();
        vault.only_owner(self.attendee())?;

        let mut requests = VaultNativeStorage::storage();
        requests.vendor_id.set(vendor_id);
        requests.custody.set(custody);
        requests.collateral_asset.set(asset);
        requests.max_order_size.set(max_order_size);

        Ok(())
    }

    pub fn set_operator(&mut self, operator: Address, status: bool) -> Result<(), Vec<u8>> {
        let vault = VaultStorage::storage();
        vault.only_owner(self.attendee())?;

        let mut requests = VaultNativeStorage::storage();
        requests.set_operator(operator, status);
        Ok(())
    }

    pub fn is_operator(&self, operator: Address) -> bool {
        let requests = VaultNativeStorage::storage();
        requests.is_operator(operator)
    }

    /// Returns asset used as collateral paying for underlying assets.
    pub fn collateral_asset(&self) -> Address {
        let requests = VaultNativeStorage::storage();
        requests.collateral_asset.get()
    }

    pub fn vendor_id(&self) -> U128 {
        let requests = VaultNativeStorage::storage();
        requests.vendor_id.get()
    }

    pub fn custody_address(&self) -> Address {
        let requests = VaultNativeStorage::storage();
        requests.custody.get()
    }

    /// Returns value of underlying assets using micro-price
    /// without applying slippage.
    pub fn assets_value(&self, account: Address) -> Result<U128, Vec<u8>> {
        let vault = VaultStorage::storage();
        let requests = VaultNativeStorage::storage();

        let order = vault.get_order(self, account)?;
        let quote = requests.get_quote(&vault, self)?;

        let itp_amount = order.tell_total()?;
        let assets_base_value = quote.tell_base_value(itp_amount)?;

        Ok(assets_base_value.to_u128())
    }

    /// Returns total value of all assets locked in this Index
    /// without applying slippage.
    pub fn total_assets_value(&self) -> Result<U128, Vec<u8>> {
        let vault = VaultStorage::storage();
        let requests = VaultNativeStorage::storage();

        let order = vault.get_total_order(self)?;
        let quote = requests.get_quote(&vault, self)?;

        let itp_amount = order.tell_total()?;
        let assets_base_value = quote.tell_base_value(itp_amount)?;

        Ok(assets_base_value.to_u128())
    }

    /// Returns value of given amount of ITP without applying slippage.
    pub fn convert_assets_value(&self, shares: U128) -> Result<U128, Vec<u8>> {
        let vault = VaultStorage::storage();
        let requests = VaultNativeStorage::storage();

        log_msg!("Getting quote");
        let quote = requests.get_quote(&vault, self)?;
        let itp_amount = Amount::from_u128(shares);
        log_msg!("Telling base value");
        let base_value = quote.tell_base_value(itp_amount)?;

        Ok(base_value.to_u128())
    }

    /// Returns amount of ITP with given value computed without slippage.
    pub fn convert_itp_amount(&self, assets: U128) -> Result<U128, Vec<u8>> {
        let vault = VaultStorage::storage();
        let requests = VaultNativeStorage::storage();

        let quote = requests.get_quote(&vault, self)?;
        let base_value = Amount::from_u128(assets);
        let itp_amount = quote.tell_itp_amount(base_value)?;

        Ok(itp_amount.to_u128())
    }

    /// Returns estimated cost of acquiring (buying) given amount of ITP.
    /// Function applies slippage and MaxOrderSize.
    pub fn estimate_acquisition_cost(&self, shares: U128) -> Result<U128, Vec<u8>> {
        let vault = VaultStorage::storage();
        let requests = VaultNativeStorage::storage();

        let quote = requests.get_quote(&vault, self)?;
        let itp_amount = Amount::from_u128(shares);
        let max_order_size = Amount::from_u128(requests.max_order_size.get());
        let base_value = quote
            .estimate_acquisition_cost(itp_amount, max_order_size)
            .ok_or_else(|| {
                format!(
                    "Failed to estimate cost: {} ITP ({})",
                    itp_amount.0, max_order_size.0
                )
            })?;

        Ok(base_value.to_u128())
    }

    /// Returns estimated amount of ITP you will get if you pay given cost.
    /// Function applies slippage and MaxOrderSize.
    pub fn estimate_acquisition_itp(&self, assets: U128) -> Result<U128, Vec<u8>> {
        let vault = VaultStorage::storage();
        let requests = VaultNativeStorage::storage();

        let quote = requests.get_quote(&vault, self)?;
        let cost = Amount::from_u128(assets);
        let max_order_size = Amount::from_u128(requests.max_order_size.get());
        let itp_amount = quote
            .estimate_acquisition_itp(cost, max_order_size)
            .ok_or_else(|| format!("Failed to estimate ITP: {} ({})", cost.0, max_order_size.0))?;

        Ok(itp_amount.to_u128())
    }

    /// Returns estimated amount of gains from selling given amount of ITP.
    /// Function applies slippage and MaxOrderSize.
    pub fn estimate_disposal_gains(&self, shares: U128) -> Result<U128, Vec<u8>> {
        let vault = VaultStorage::storage();
        let requests = VaultNativeStorage::storage();

        let quote = requests.get_quote(&vault, self)?;
        let itp_amount = Amount::from_u128(shares);
        let max_order_size = Amount::from_u128(requests.max_order_size.get());
        let base_value = quote
            .estimate_disposal_gains(itp_amount, max_order_size)
            .ok_or_else(|| {
                format!(
                    "Failed to estimate cost: {} ITP ({})",
                    itp_amount.0, max_order_size.0
                )
            })?;

        Ok(base_value.to_u128())
    }

    /// Returns estimated amount of ITP you need to sell to obtain given gains.
    /// Function applies slippage and MaxOrderSize.
    pub fn estimate_disposal_itp_cost(&self, assets: U128) -> Result<U128, Vec<u8>> {
        let vault = VaultStorage::storage();
        let requests = VaultNativeStorage::storage();

        let quote = requests.get_quote(&vault, self)?;
        let gains = Amount::from_u128(assets);
        let max_order_size = Amount::from_u128(requests.max_order_size.get());
        let itp_amount = quote
            .estimate_disposal_itp_cost(gains, max_order_size)
            .ok_or_else(|| {
                format!(
                    "Failed to estimate ITP cost: {} ({})",
                    gains.0, max_order_size.0
                )
            })?;

        Ok(itp_amount.to_u128())
    }

    /// Returns MaxOrderSize before order is split into multiple chunks.
    ///
    /// When order size (measured in collateral asset) is less than
    /// MaxOrderSize, then that order can be filled instantly. This is subject
    /// to current CapacityLimit, which changes dynamically as new orders come,
    /// supply, or market data changes.
    ///
    pub fn get_max_order_size(&self) -> U128 {
        let requests = VaultNativeStorage::storage();
        requests.max_order_size.get()
    }

    /// Returns (Capacity, Price, Slope) tuple
    pub fn get_quote(&self) -> Result<(U128, U128, U128), Vec<u8>> {
        let vault = VaultStorage::storage();
        let requests = VaultNativeStorage::storage();

        let quote = requests.get_quote(&vault, self)?;

        Ok((
            quote.capacity().to_u128(),
            quote.price().to_u128(),
            quote.slope().to_u128(),
        ))
    }

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
        trader: Address,
    ) -> Result<(), Vec<u8>> {
        if collateral_amount.is_zero() {
            Err(b"Zero collateral amount")?;
        }
        let vault = VaultStorage::storage();
        let requests = VaultNativeStorage::storage();
        let sender = self.attendee();

        if sender != trader {
            Err(b"Sender must be an owner")?;
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
        let request_event = if instant_fill {
            // We should use fresh prices
            requests.update_quote(&vault, self)?;

            self.external_call(
                vault.gate_to_castle.get(),
                IFactor::executeBuyOrderCall {
                    vendor_id: requests.vendor_id.get().to(),
                    index_id: vault.index_id.get().to(),
                    trader_address: trader,
                    collateral_added: collateral_amount.to(),
                    collateral_removed: 0,
                    max_order_size: requests.max_order_size.get().to(),
                },
            )?;
            // We publish event with zero collateral added, as we already
            // updated our order in Clerk Chamber, and if fill was only partial,
            // then Keeper needs to continue filling.
            BuyOrder {
                index_id: vault.index_id.get().to(),
                vendor_id: requests.vendor_id.get().to(),
                collateral_amount: 0,
                trader,
            }
        } else {
            // Send pending order without executing it
            self.external_call(
                vault.gate_to_castle.get(),
                IFactor::submitBuyOrderCall {
                    vendor_id: requests.vendor_id.get().to(),
                    index_id: vault.index_id.get().to(),
                    trader_address: trader,
                    collateral_added: collateral_amount.to(),
                    collateral_removed: 0,
                },
            )?;

            // We publish event with original collateral amount, as we only
            // deposited collateral, but haven't executed anything yet. Keeper
            // service will call submitByOrder() on our behalf.
            BuyOrder {
                index_id: vault.index_id.get().to(),
                vendor_id: requests.vendor_id.get().to(),
                collateral_amount: collateral_amount.to(),
                trader,
            }
        };

        // Send an event, and it will be picked up by Keeper service
        self.vm().emit_log(&request_event.encode_data(), 1);

        Ok(())
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
        trader: Address,
    ) -> Result<(), Vec<u8>> {
        if itp_amount.is_zero() {
            Err(b"Zero ITP amount")?;
        }

        let vault = VaultStorage::storage();
        let requests = VaultNativeStorage::storage();
        let sender = self.attendee();

        if sender != trader {
            Err(b"Sender must be an owner")?;
        }

        let request_event = if instant_fill {
            // We should use fresh prices
            requests.update_quote(&vault, self)?;

            // Submit order and get instant fill if possible
            self.external_call(
                vault.gate_to_castle.get(),
                IFactor::executeSellOrderCall {
                    vendor_id: requests.vendor_id.get().to(),
                    index_id: vault.index_id.get().to(),
                    trader_address: trader,
                    collateral_added: itp_amount.to(),
                    collateral_removed: 0,
                    max_order_size: requests.max_order_size.get().to(),
                },
            )?;
            // We publish event with zero ITP added, as we already updated our
            // order in Clerk Chamber, and if fill was only partial, then Keeper
            // needs to continue filling.
            SellOrder {
                index_id: vault.index_id.get().to(),
                vendor_id: requests.vendor_id.get().to(),
                itp_amount: 0,
                trader,
            }
        } else {
            // Send pending order without executing it
            self.external_call(
                vault.gate_to_castle.get(),
                IFactor::submitSellOrderCall {
                    vendor_id: requests.vendor_id.get().to(),
                    index_id: vault.index_id.get().to(),
                    trader_address: trader,
                    collateral_added: itp_amount.to(),
                    collateral_removed: 0,
                },
            )?;

            // We publish event with original ITP amount, as we haven't executed
            // anything yet. Keeper service will call submitByOrder() on our
            // behalf.
            SellOrder {
                index_id: vault.index_id.get().to(),
                vendor_id: requests.vendor_id.get().to(),
                itp_amount: itp_amount.to(),
                trader,
            }
        };

        // Send an event, and it will be picked up by Keeper service.
        self.vm().emit_log(&request_event.encode_data(), 1);

        Ok(())
    }

    /// Keeper confirms that portion of SELL order has been realized and amount
    /// of gains is available for withdrawal.
    pub fn confirm_withdraw_available(
        &mut self,
        amount: U128,
        trader: Address,
    ) -> Result<(), Vec<u8>> {
        let mut requests = VaultNativeStorage::storage();
        let sender = self.attendee();

        if !requests.is_operator(sender) {
            Err(b"Sender must be an operator")?;
        }

        // Update our records of withdraw ready, as now we have confirmation
        // that Keeper has seen BuyOrder event, and collateral is part of the
        // active order.
        let mut trader_order = requests.trader_orders.setter(trader);
        let mut withdraw_ready = trader_order.withdraw_ready.get();
        withdraw_ready = withdraw_ready
            .checked_add(amount)
            .ok_or_else(|| b"MathOverflow")?;
        trader_order.withdraw_ready.set(withdraw_ready);

        Ok(())
    }

    /// Withdraw gains produced from selling ITP.
    ///
    /// When we submit SELL order the balance of USDC remains stored in Clerk
    /// Chamber, and we also store amount we have withdrawn so far, so we can
    /// claim any difference. Function returns amount claimed.
    ///
    /// Assumption is that custody has pre-approved that amount to be withdrawn.
    /// This approval should have happened off-chain, when Keeper service
    /// completed SELL order.
    ///
    pub fn withdraw_gains(&mut self, amount: U128, trader: Address) -> Result<(), Vec<u8>> {
        let mut requests = VaultNativeStorage::storage();
        let sender = self.attendee();

        if sender != trader {
            Err(b"Sender must be an owner")?;
        }

        let mut trader_order = requests.trader_orders.setter(trader);
        let mut withdraw_ready = trader_order.withdraw_ready.get();

        withdraw_ready = withdraw_ready
            .checked_sub(amount)
            .ok_or_else(|| b"Insufficient funds ready")?;

        trader_order.withdraw_ready.set(withdraw_ready);

        let asset = requests.collateral_asset.get();
        self.external_call(
            asset,
            IERC20::transferFromCall {
                from: requests.custody.get(),
                to: trader,
                value: amount.to(),
            },
        )?;

        Ok(())
    }

    /// USDC available for withdrawal.
    pub fn get_withdraw_available(&self, trader: Address) -> Result<U128, Vec<u8>> {
        let requests = VaultNativeStorage::storage();
        let trader_order = requests.trader_orders.getter(trader);
        let withdraw_ready = trader_order.withdraw_ready.get();
        Ok(withdraw_ready)
    }

    /// Collateral for BUY order that was accepted by Keeper.
    pub fn get_active_acquisition_collateral(&self, trader: Address) -> Result<U128, Vec<u8>> {
        let vault = VaultStorage::storage();
        let order = vault.get_order(self, trader)?;
        Ok(order.collateral_remaining().to_u128())
    }

    /// ITP for SELL order that was accepted by Keeper.
    pub fn get_active_disposal_itp(&self, trader: Address) -> Result<U128, Vec<u8>> {
        let vault = VaultStorage::storage();
        let order = vault.get_order(self, trader)?;
        Ok(order.itp_locked().to_u128())
    }
}
