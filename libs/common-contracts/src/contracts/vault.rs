use alloc::{vec, vec::Vec};

use alloy_primitives::{uint, Address, U256, U32};
use stylus_sdk::{
    keccak_const,
    prelude::*,
    storage::{StorageAddress, StorageMap, StorageString, StorageU128, StorageU256, StorageU32},
};

use crate::{
    contracts::{calls::InnerCall, formulas::Order, storage::StorageSlot},
    interfaces::steward::ISteward,
};

pub const VAULT_STORAGE_SLOT: U256 = {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"Vault.STORAGE_SLOT")
        .finalize();
    U256::from_be_bytes(HASH).wrapping_sub(uint!(1_U256))
};

#[storage]
pub struct Allowance {
    from_account: StorageMap<Address, StorageU256>,
}

impl Allowance {
    pub fn allowance(&self, spender: Address) -> U256 {
        self.from_account.get(spender)
    }

    pub fn approve(&mut self, spender: Address, value: U256) -> Result<bool, Vec<u8>> {
        if spender.is_zero() {
            Err(b"Invalid Spender")?;
        }
        let mut allowance = self.from_account.setter(spender);
        allowance.set(value);
        Ok(true)
    }

    pub fn spend_allowance(&mut self, spender: Address, value: U256) -> Result<(), Vec<u8>> {
        if spender.is_zero() {
            Err(b"Invalid Spender")?;
        }
        let mut allowance = self.from_account.setter(spender);
        let current = allowance.get();
        let remain = current
            .checked_sub(value)
            .ok_or_else(|| b"Insufficient Allowance")?;
        allowance.set(remain);
        Ok(())
    }
}

#[storage]
pub struct VaultStorage {
    pub version: StorageU32,
    pub index_id: StorageU128,
    pub owner: StorageAddress,
    pub castle: StorageAddress,
    // balance & allowance
    pub total_supply: StorageU256,
    pub balances: StorageMap<Address, StorageU256>,
    pub allowances: StorageMap<Address, Allowance>,
    // facets
    pub vault_implementation: StorageAddress,
    pub orders_implementation: StorageAddress,
    pub claims_implementation: StorageAddress,
    // detail
    pub name: StorageString,
    pub symbol: StorageString,
    pub description: StorageString,
    pub methodology: StorageString,
    pub initial_price: StorageU128,
    pub curator: StorageAddress,
    pub custody: StorageString,
}

impl VaultStorage {
    pub fn storage() -> VaultStorage {
        StorageSlot::get_slot::<VaultStorage>(VAULT_STORAGE_SLOT)
    }

    pub fn only_owner(&self, sender: Address) -> Result<(), Vec<u8>> {
        let owner = self.owner.get();
        if !owner.is_zero() && owner != sender {
            Err(b"Only owner")?;
        }
        Ok(())
    }

    pub fn set_version(&mut self, version: U32) -> Result<(), Vec<u8>> {
        if self.version.get() > version {
            Err(b"Version cannot be downgraded")?;
        }
        self.version.set(version);
        Ok(())
    }

    pub fn set_owner(&mut self, new_owner: Address) -> Result<(), Vec<u8>> {
        self.owner.set(new_owner);
        Ok(())
    }

    pub fn set_vault_implementation(&mut self, vault_implementation: Address) {
        self.vault_implementation.set(vault_implementation);
    }

    pub fn set_orders_implementation(&mut self, orders_implementation: Address) {
        self.orders_implementation.set(orders_implementation);
    }

    pub fn set_claims_implementation(&mut self, claims_implementation: Address) {
        self.claims_implementation.set(claims_implementation);
    }

    pub fn set_castle(&mut self, gate_to_castle: Address) {
        self.castle.set(gate_to_castle);
    }

    fn _add_balance(&mut self, to: Address, amount: U256) -> Result<(), Vec<u8>> {
        let mut balance = self.balances.setter(to);
        let current_balance = balance.get();
        let new_balance = current_balance
            .checked_add(amount)
            .ok_or_else(|| b"MathOverflow (balance + amount)")?;
        balance.set(new_balance);
        Ok(())
    }

    fn _reduce_balance(&mut self, to: Address, amount: U256) -> Result<(), Vec<u8>> {
        let mut balance = self.balances.setter(to);
        let current_balance = balance.get();
        let new_balance = current_balance
            .checked_sub(amount)
            .ok_or_else(|| b"MathOverflow (balance - amount)")?;
        balance.set(new_balance);
        Ok(())
    }

    pub fn mint(&mut self, to: Address, amount: U256) -> Result<(), Vec<u8>> {
        let current_supply = self.total_supply.get();
        let new_supply = current_supply
            .checked_add(amount)
            .ok_or_else(|| b"MathOverflow (total_supply + amount)")?;
        self.total_supply.set(new_supply);
        self._add_balance(to, amount)
    }

    pub fn burn(&mut self, from: Address, amount: U256) -> Result<(), Vec<u8>> {
        let current_supply = self.total_supply.get();
        let new_supply = current_supply
            .checked_sub(amount)
            .ok_or_else(|| b"MathOverflow (total_supply - amount)")?;
        self.total_supply.set(new_supply);
        self._reduce_balance(from, amount)
    }

    pub fn transfer(&mut self, from: Address, to: Address, amount: U256) -> Result<(), Vec<u8>> {
        self._reduce_balance(from, amount)?;
        self._add_balance(to, amount)?;
        Ok(())
    }

    pub fn balance_of(&self, account: Address) -> U256 {
        self.balances.get(account)
    }

    pub fn get_total_supply(&self) -> U256 {
        self.total_supply.get()
    }

    pub fn get_order(&self, caller: &impl InnerCall, account: Address) -> Result<Order, Vec<u8>> {
        let call = ISteward::getTraderOrderCall {
            index_id: self.index_id.get().to(),
            trader: account,
        };
        let ISteward::getTraderOrderReturn { _0: ret } =
            caller.static_call_ret(self.castle.get(), call)?;

        let order = Order::try_from_vec(ret.into()).map_err(|_| b"Failed to decode order data")?;
        Ok(order)
    }

    pub fn get_total_order(&self, caller: &impl InnerCall) -> Result<Order, Vec<u8>> {
        let call = ISteward::getTotalOrderCall {
            index_id: self.index_id.get().to(),
        };
        let ISteward::getTotalOrderReturn { _0: ret } =
            caller.static_call_ret(self.castle.get(), call)?;

        let order = Order::try_from_vec(ret.into()).map_err(|_| b"Failed to decode order data")?;
        Ok(order)
    }
}
