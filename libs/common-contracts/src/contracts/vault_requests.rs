use alloc::{vec, vec::Vec};

use alloy_primitives::{uint, Address, U256};
use common::amount::Amount;
use stylus_sdk::{
    keccak_const,
    prelude::*,
    storage::{StorageAddress, StorageBool, StorageMap, StorageU128},
};

use crate::{
    contracts::{calls::InnerCall, formulas::Quote, storage::StorageSlot, vault::VaultStorage},
    interfaces::steward::ISteward,
};

pub const VAULT_REQUESTS_STORAGE_SLOT: U256 = {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"VaultRequests.STORAGE_SLOT")
        .finalize();
    U256::from_be_bytes(HASH).wrapping_sub(uint!(1_U256))
};

#[storage]
pub struct Request {
    pending_request: StorageU128,
    claimable_request: StorageU128,
    claimable_amount: StorageU128,
}

impl Request {
    pub fn pending(&self) -> Amount {
        Amount::from_u128(self.pending_request.get())
    }

    pub fn claimable(&self) -> Amount {
        Amount::from_u128(self.claimable_request.get())
    }

    pub fn request(&mut self, amount: Amount) -> Result<(), Vec<u8>> {
        let current = Amount::from_u128(self.pending_request.get());
        let result = current.checked_add(amount).ok_or_else(|| b"MathOverflow")?;
        self.pending_request.set(result.to_u128());
        Ok(())
    }

    pub fn update(&mut self, spent: Amount, ready: Amount) -> Result<Amount, Vec<u8>> {
        let pending = Amount::from_u128(self.pending_request.get());
        let claimable = Amount::from_u128(self.claimable_request.get());
        let amount = Amount::from_u128(self.claimable_amount.get());

        let pending_new = pending.checked_sub(spent).ok_or_else(|| b"MathOverflow")?;
        let claimable_new = claimable
            .checked_add(spent)
            .ok_or_else(|| b"MathOverflow")?;
        let amount_new = amount.checked_add(ready).ok_or_else(|| b"MathOverflow")?;

        self.pending_request.set(pending_new.to_u128());
        self.claimable_request.set(claimable_new.to_u128());
        self.claimable_amount.set(amount_new.to_u128());

        Ok(amount_new)
    }

    pub fn claim(&mut self, amount: Amount) -> Result<Amount, Vec<u8>> {
        let current = Amount::from_u128(self.claimable_request.get());
        let claimable = Amount::from_u128(self.claimable_amount.get());

        let to_claim = if amount == claimable {
            // use total claimable
            claimable
        } else {
            // distribute pro-rata
            claimable
                .checked_mul(amount)
                .and_then(|x| x.checked_div(current))
                .ok_or_else(|| b"MathOverflow")?
        };

        let current_new = current
            .checked_sub(amount)
            .ok_or_else(|| b"Insufficient Claimable")?;

        let claimable_new = claimable
            .checked_sub(to_claim)
            .ok_or_else(|| b"Insufficient Claimable")?;

        self.claimable_request.set(current_new.to_u128());
        self.claimable_amount.set(claimable_new.to_u128());

        Ok(to_claim)
    }
}

#[storage]
pub struct Requests {
    active: StorageBool,
    request: Request,
}

impl Requests {
    pub fn is_active(&self) -> bool {
        self.active.get()
    }

    pub fn pending(&self, _: U256) -> Amount {
        self.request.pending()
    }

    pub fn claimable(&self, _: U256) -> Amount {
        self.request.claimable()
    }

    pub fn request(&mut self, amount: Amount) -> Result<U256, Vec<u8>> {
        self.active.set(true);
        self.request.request(amount)?;
        Ok(U256::ZERO)
    }

    pub fn update(&mut self, _: U256, spent: Amount, ready: Amount) -> Result<Amount, Vec<u8>> {
        self.request.update(spent, ready)
    }

    pub fn claim(&mut self, amount: Amount) -> Result<Amount, Vec<u8>> {
        let claimable = self.request.claimable();

        let amount_remain = amount
            .saturating_sub(claimable)
            .ok_or_else(|| b"UnexpectedMathError")?;

        if amount_remain.is_zero() {
            let to_claim = claimable
                .checked_sub(amount)
                .ok_or_else(|| b"UnexpectedMathError")?;

            return self.request.claim(to_claim);
        }

        Err(b"Insufficient Claimable".into())
    }
}

#[storage]
pub struct Operator {
    operators: StorageMap<Address, StorageBool>,
}

impl Operator {
    pub fn is_operator(&self, operator: Address) -> bool {
        self.operators.get(operator)
    }

    pub fn set_operator(&mut self, operator: Address, approved: bool) {
        let mut setter = self.operators.setter(operator);
        setter.set(approved);
    }
}

#[storage]
pub struct VaultRequestsStorage {
    pub vendor_id: StorageU128,
    pub custody: StorageAddress,
    pub collateral_asset: StorageAddress,
    pub operators: StorageMap<Address, Operator>,
    pub deposit_request: StorageMap<Address, Requests>,
    pub redeem_request: StorageMap<Address, Requests>,
}

impl VaultRequestsStorage {
    pub fn storage() -> VaultRequestsStorage {
        StorageSlot::get_slot::<VaultRequestsStorage>(VAULT_REQUESTS_STORAGE_SLOT)
    }

    pub fn get_quote(
        &self,
        vault: &VaultStorage,
        caller: &impl InnerCall,
    ) -> Result<Quote, Vec<u8>> {
        let call = ISteward::getIndexQuoteCall {
            index_id: vault.index_id.get().to(),
            vendor_id: self.vendor_id.get().to(),
        };
        let ISteward::getIndexQuoteReturn { _0: ret } =
            caller.static_call_ret(vault.gate_to_castle.get(), call)?;

        let quote = Quote::try_from_vec(ret.into()).map_err(|_| b"Failed to decode quote data")?;
        Ok(quote)
    }
}
