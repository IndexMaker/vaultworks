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
    contracts::{keep_calls::KeepCalls, vault::VaultStorage, vault_native::VaultNativeStorage},
    interfaces::vault_native::IVaultNative::OperatorSet,
};
use stylus_sdk::{prelude::*, ArbResult};

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
    pub fn install_orders(&mut self, orders_implementation: Address) -> Result<(), Vec<u8>> {
        let vault = VaultStorage::storage();
        vault.only_owner(self.attendee())?;

        let mut requests = VaultNativeStorage::storage();
        requests.orders_implementation.set(orders_implementation);
        Ok(())
    }

    pub fn install_claims(&mut self, claims_implementation: Address) -> Result<(), Vec<u8>> {
        let vault = VaultStorage::storage();
        vault.only_owner(self.attendee())?;

        let mut requests = VaultNativeStorage::storage();
        requests.claims_implementation.set(claims_implementation);
        Ok(())
    }

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

    pub fn is_operator(&self, owner: Address, operator: Address) -> bool {
        let requests = VaultNativeStorage::storage();
        requests.is_operator(owner, operator)
    }

    pub fn set_operator(&mut self, operator: Address, approved: bool) -> bool {
        let sender = self.attendee();
        let mut requests = VaultNativeStorage::storage();
        let mut operators = requests.operators.setter(sender);
        operators.set_operator(operator, approved);

        let event = OperatorSet {
            controller: sender,
            operator,
            approved,
        };

        self.vm().emit_log(&event.encode_data(), 1);
        true
    }

    pub fn set_admin_operator(&mut self, controller: Address, approved: bool) -> Result<(), Vec<u8>> {
        let sender = self.attendee();
        let vault = VaultStorage::storage();
        vault.only_owner(sender)?;

        let mut requests = VaultNativeStorage::storage();
        let mut operators = requests.operators.setter(controller);
        operators.set_operator(sender, approved);

        let event = OperatorSet {
            controller,
            operator: sender,
            approved,
        };

        self.vm().emit_log(&event.encode_data(), 1);
        Ok(())
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

    #[payable]
    #[fallback]
    fn fallback(&mut self, calldata: &[u8]) -> ArbResult {
        let requests = {
            let requests = VaultNativeStorage::storage();
            let implementation = requests.orders_implementation.get();
            if implementation.is_zero() {
                Err(b"No orders implementation")?;
            }
            implementation
        };

        unsafe {
            let result = self.vm().delegate_call(&self, requests, calldata)?;
            Ok(result)
        }
    }
}
