// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U128};
use alloy_sol_types::{sol, SolEvent};
use common::amount::Amount;
use common_contracts::{
    contracts::{
        calls::InnerCall, keep_calls::KeepCalls, vault::VaultStorage,
        vault_native::VaultNativeStorage,
    },
    interfaces::vault_native_claims::IVaultNativeClaims::{AcquisitionClaim, DisposalClaim},
};
use stylus_sdk::prelude::*;

use crate::IERC20::transferFromCall;

sol! {
    interface IERC20 {
        function transferFrom(address from, address to, uint256 value) external returns (bool);
    }
}

#[storage]
#[entrypoint]
pub struct VaultNativeClaims;

#[public]
impl VaultNativeClaims {
    /// Tell pending order for trader and keeper (keeper's liability towards trader)
    pub fn get_pending_order(
        &self,
        keeper: Address,
        trader: Address,
    ) -> Result<(U128, U128), Vec<u8>> {
        let requests = VaultNativeStorage::storage();
        let getter = requests.trader_orders.getter(trader);
        Ok((
            getter.pending_bid.get(keeper),
            getter.pending_ask.get(keeper),
        ))
    }

    /// Tell claimable ITP available and the cost
    pub fn get_claimable_acquisition(&self, keeper: Address) -> Result<(U128, U128), Vec<u8>> {
        let requests = VaultNativeStorage::storage();
        let getter = requests.opearator_order.getter(keeper);
        Ok((getter.bid_received.get(), getter.bid_delivered.get()))
    }

    /// Tell total claimable gains and amount of ITP burned
    pub fn get_claimable_disposal(&self, keeper: Address) -> Result<(U128, U128), Vec<u8>> {
        let requests = VaultNativeStorage::storage();
        let getter = requests.opearator_order.getter(keeper);
        Ok((getter.ask_received.get(), getter.ask_delivered.get()))
    }

    /// Pay part of the ITP acquisition cost to claim ITP.
    pub fn claim_acquisition(
        &mut self,
        collateral_amount: U128,
        keeper: Address,
        trader: Address,
    ) -> Result<U128, Vec<u8>> {
        let vault = VaultStorage::storage();
        let mut requests = VaultNativeStorage::storage();
        let sender = self.attendee();

        if sender != keeper && !requests.is_operator(keeper, sender) {
            Err(b"Unauthorised order processing")?;
        }

        let mut keeper_order = requests.opearator_order.setter(keeper);
        let mut trader_order = requests.trader_orders.setter(trader);
        let mut pending_bid = trader_order.pending_bid.setter(keeper);

        let collateral_spent = keeper_order.bid_delivered.get();
        let itp_received = keeper_order.bid_received.get();

        let itp_claimed = Amount::from_u128(collateral_amount)
            .checked_mul(Amount::from_u128(itp_received))
            .ok_or_else(|| b"MathOverflow")?
            .checked_div(Amount::from_u128(collateral_spent))
            .ok_or_else(|| b"MathOverflow")?
            .to_u128();

        // Transfer ITP from keeper to Trader
        self.external_call(
            vault.gate_to_castle.get(),
            transferFromCall {
                from: keeper,
                to: trader,
                value: itp_claimed.to(),
            },
        )?;

        let collateral_spent = collateral_spent
            .checked_sub(collateral_amount)
            .ok_or_else(|| b"limit exceeded")?;

        let itp_received = itp_received
            .checked_sub(itp_claimed)
            .ok_or_else(|| b"Limit exceeded")?;

        let pending_amount = pending_bid
            .get()
            .checked_sub(collateral_amount)
            .ok_or_else(|| b"Limit exceeded")?;

        keeper_order.bid_delivered.set(collateral_spent);
        keeper_order.bid_received.set(itp_received);
        pending_bid.set(pending_amount);

        if !itp_claimed.is_zero() {
            // Publish execution report if there was execution

            let exec_report = AcquisitionClaim {
                controller: keeper,
                index_id: vault.index_id.get().to(),
                vendor_id: requests.vendor_id.get().to(),
                remain: pending_amount.to(),
                spent: collateral_amount.to(),
                itp_minted: itp_claimed.to(),
            };

            self.vm().emit_log(&exec_report.encode_data(), 1);
        }

        Ok(itp_claimed)
    }

    /// Claim gains from ITP disposal.
    pub fn claim_disposal(
        &mut self,
        itp_amount: U128,
        keeper: Address,
        trader: Address,
    ) -> Result<U128, Vec<u8>> {
        let vault = VaultStorage::storage();
        let mut requests = VaultNativeStorage::storage();
        let sender = self.attendee();

        if sender != keeper && !requests.is_operator(keeper, sender) {
            Err(b"Unauthorised order processing")?;
        }

        let mut keeper_order = requests.opearator_order.setter(keeper);
        let mut trader_order = requests.trader_orders.setter(trader);
        let mut pending_ask = trader_order.pending_ask.setter(keeper);

        let itp_burned = keeper_order.ask_delivered.get();
        let amount_received = keeper_order.ask_received.get();

        let amount_claimed = Amount::from_u128(itp_amount)
            .checked_mul(Amount::from_u128(amount_received))
            .ok_or_else(|| b"MathOverflow")?
            .checked_div(Amount::from_u128(itp_burned))
            .ok_or_else(|| b"MathOverflow")?
            .to_u128();

        // Tranfer gains from keeper to Trader
        self.external_call(
            requests.collateral_asset.get(),
            transferFromCall {
                from: keeper,
                to: trader,
                value: amount_claimed.to(),
            },
        )?;

        let itp_burned = itp_burned
            .checked_sub(itp_amount)
            .ok_or_else(|| b"Limit exceeded")?;

        let amount_received = amount_received
            .checked_sub(amount_claimed)
            .ok_or_else(|| b"Limit exceeded")?;

        let pending_amount = pending_ask
            .get()
            .checked_sub(itp_amount)
            .ok_or_else(|| b"Limit exceeded")?;

        keeper_order.ask_delivered.set(itp_burned);
        keeper_order.ask_received.set(amount_received);
        pending_ask.set(pending_amount);

        if !itp_amount.is_zero() {
            // Publish execution report if there was execution

            let exec_report = DisposalClaim {
                controller: keeper,
                index_id: vault.index_id.get().to(),
                vendor_id: requests.vendor_id.get().to(),
                itp_remain: pending_amount.to(),
                itp_burned: itp_amount.to(),
                gains: amount_received.to(),
            };

            self.vm().emit_log(&exec_report.encode_data(), 1);
        }

        Ok(amount_received)
    }
}
