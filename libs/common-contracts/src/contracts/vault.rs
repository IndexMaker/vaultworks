use alloc::{vec, vec::Vec};

use alloy_primitives::{uint, Address, U256};
use common::amount::Amount;
use stylus_sdk::{
    keccak_const,
    prelude::*,
    storage::{
        StorageAddress, StorageBool, StorageMap, StorageString, StorageU128, StorageU256,
        StorageU32,
    },
};

pub const VAULT_STORAGE_SLOT: U256 = {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"Vault.STORAGE_SLOT")
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
    last_claimed_id: StorageU256,
    last_request_id: StorageU256,
    total_claimable: StorageU128,
    requests: StorageMap<U256, Request>,
}

impl Requests {
    pub fn is_active(&self) -> bool {
        self.last_request_id.get().is_zero()
    }

    pub fn pending(&self, request_id: U256) -> Amount {
        self.requests.get(request_id).pending()
    }

    pub fn claimable(&self, request_id: U256) -> Amount {
        self.requests.get(request_id).claimable()
    }

    pub fn request(&mut self, amount: Amount) -> Result<U256, Vec<u8>> {
        let last_id = self.last_request_id.get();
        let new_id = last_id
            .checked_add(U256::ONE)
            .ok_or_else(|| b"MathOverflow")?;
        self.last_request_id.set(new_id);
        let mut setter = self.requests.setter(last_id);
        setter.request(amount)?;
        Ok(last_id)
    }

    pub fn update(
        &mut self,
        request_id: U256,
        spent: Amount,
        ready: Amount,
    ) -> Result<Amount, Vec<u8>> {
        let mut request = self.requests.setter(request_id);
        request.update(spent, ready)
    }

    pub fn claim(&mut self, mut amount: Amount) -> Result<Amount, Vec<u8>> {
        let total_claimable = Amount::from_u128(self.total_claimable.get());
        if total_claimable < amount {
            Err(b"Insufficient Claimable")?;
        }
        let mut first_id = self.last_claimed_id.get();
        let last_id = self.last_request_id.get();
        let mut total_claimed = Amount::ZERO;

        while first_id <= last_id {
            let mut request = self.requests.setter(first_id);
            let claimable = request.claimable();

            let amount_remain = amount
                .saturating_sub(claimable)
                .ok_or_else(|| b"UnexpectedMathError")?;

            if amount_remain.is_zero() {
                let to_claim = claimable
                    .checked_sub(amount)
                    .ok_or_else(|| b"UnexpectedMathError")?;

                let claimed = request.claim(to_claim)?;
                total_claimed = total_claimed
                    .checked_add(claimed)
                    .ok_or_else(|| b"MathOverflow")?;

                self.last_claimed_id.set(first_id);

                return Ok(total_claimed);
            } else {
                let claimed = request.claim(claimable)?;

                total_claimed = total_claimed
                    .checked_add(claimed)
                    .ok_or_else(|| b"MathOverflow")?;

                amount = amount_remain;

                first_id = first_id
                    .checked_add(U256::ONE)
                    .ok_or_else(|| b"MathOverflow")?;
            }
        }

        Err(b"Insufficient Claimable".into())
    }
}

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
pub struct VaultStorage {
    pub index_id: StorageU128,
    pub vendor_id: StorageU128,
    pub version: StorageU32,
    pub name: StorageString,
    pub symbol: StorageString,
    pub owner: StorageAddress,
    pub custody: StorageAddress,
    pub collateral_asset: StorageAddress,
    pub operators: StorageMap<Address, Operator>,
    pub allowances: StorageMap<Address, Allowance>,
    pub deposit_request: StorageMap<Address, Requests>,
    pub redeem_request: StorageMap<Address, Requests>,
    pub gate_to_castle: StorageAddress,
}
