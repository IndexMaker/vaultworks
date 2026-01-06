use alloc::{vec, vec::Vec};

use alloy_primitives::{uint, Address, U256, U32};
use stylus_sdk::{
    keccak_const,
    prelude::*,
    storage::{StorageAddress, StorageMap, StorageString, StorageU128, StorageU256, StorageU32},
};

use crate::{
    contracts::{
        calls::InnerCall,
        formulas::Order,
        storage::StorageSlot,
    },
    interfaces::factor::IFactor,
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
    pub index_id: StorageU128,
    pub name: StorageString,
    pub symbol: StorageString,
    pub version: StorageU32,
    pub owner: StorageAddress,
    pub allowances: StorageMap<Address, Allowance>,
    pub requests_implementation: StorageAddress,
    pub gate_to_castle: StorageAddress,
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

    pub fn set_requests(&mut self, requests: Address) {
        self.requests_implementation.set(requests);
    }

    pub fn set_castle(&mut self, gate_to_castle: Address) {
        self.gate_to_castle.set(gate_to_castle);
    }

    pub fn get_order(&self, caller: &impl InnerCall, account: Address) -> Result<Order, Vec<u8>> {
        let call = IFactor::getTraderOrderCall {
            index_id: self.index_id.get().to(),
            trader: account,
        };
        let IFactor::getTraderOrderReturn { _0: ret } =
            caller.static_call_ret(self.gate_to_castle.get(), call)?;

        let order = Order::try_from_vec(ret).map_err(|_| b"Failed to decode order data")?;
        Ok(order)
    }

    pub fn get_total_order(&self, caller: &impl InnerCall) -> Result<Order, Vec<u8>> {
        let call = IFactor::getTotalOrderCall {
            index_id: self.index_id.get().to(),
        };
        let IFactor::getTotalOrderReturn { _0: ret } =
            caller.static_call_ret(self.gate_to_castle.get(), call)?;

        let order = Order::try_from_vec(ret).map_err(|_| b"Failed to decode order data")?;
        Ok(order)
    }
}
