// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::{string::String, vec::Vec};

use alloy_primitives::{Address, U8, U32, U128, U256, uint};
use common::{amount::Amount, vector::Vector};
use common_contracts::{
    contracts::{calls::InnerCall, storage::StorageSlot},
    interfaces::factor::IFactor,
};
use stylus_sdk::{
    abi::Bytes, keccak_const, prelude::*, storage::{StorageAddress, StorageString, StorageU32, StorageU128}
};

pub const VAULT_STORAGE_SLOT: U256 = {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"Vault.STORAGE_SLOT")
        .finalize();
    U256::from_be_bytes(HASH).wrapping_sub(uint!(1_U256))
};

pub const VERSION_NUMBER: U32 = uint!(1_U32);

const ORDER_REMAIN_OFFSET: usize = 0;
const ORDER_SPENT_OFFSET: usize = 1;
const ORDER_REALIZED_OFFSET: usize = 2;

const QUOTE_CAPACITY_OFFSET: usize = 0;
const QUOTE_PRICE_OFFSET: usize = 1;
const QUOTE_SLOPE_OFFSET: usize = 2;

#[storage]
struct VaultStorage {
    index_id: StorageU128,
    vendor_id: StorageU128,
    version: StorageU32,
    name: StorageString,
    symbol: StorageString,
    gate_to_castle: StorageAddress,
}

#[storage]
#[entrypoint]
pub struct Vault;

impl Vault {
    fn _storage() -> VaultStorage {
        StorageSlot::get_slot::<VaultStorage>(VAULT_STORAGE_SLOT)
    }
}

#[public]
impl Vault {
    #[constructor]
    fn constructor(&mut self) {}

    pub fn name(&self) -> alloc::string::String {
        let vault = Self::_storage();
        vault.name.get_string()
    }

    pub fn symbol(&self) -> alloc::string::String {
        let vault = Self::_storage();
        vault.symbol.get_string()
    }

    pub fn decimals(&self) -> U8 {
        U8::from(18)
    }

    pub fn balance_of(&self, account: Address) -> Result<U128, Vec<u8>> {
        let vault = Self::_storage();

        let ret = self.static_call_ret(
            vault.gate_to_castle.get(),
            IFactor::getTraderOrderCall {
                index_id: vault.index_id.get().to(),
                trader: account,
            },
        )?;

        let bid = Vector::from_vec(ret._0);
        let ask = Vector::from_vec(ret._1);

        let itp_available = bid.data[ORDER_REALIZED_OFFSET]
            .checked_sub(ask.data[ORDER_REMAIN_OFFSET])
            .ok_or_else(|| b"MathUnderflow")?;

        Ok(itp_available.to_u128())
    }

    pub fn assets(&self, account: Address) -> Result<U128, Vec<u8>> {
        let vault = Self::_storage();

        let ret1 = self.static_call_ret(
            vault.gate_to_castle.get(),
            IFactor::getTraderOrderCall {
                index_id: vault.index_id.get().to(),
                trader: account,
            },
        )?;

        let ret2 = self.static_call(
            vault.gate_to_castle.get(),
            IFactor::getIndexQuoteCall {
                index_id: vault.index_id.get().to(),
                vendor_id: vault.vendor_id.get().to(),
            },
        )?;

        let bid = Vector::from_vec(ret1._0);
        let ask = Vector::from_vec(ret1._1);
        let quote = Vector::from_vec(ret2);

        let itp_unburnt = bid.data[ORDER_REALIZED_OFFSET]
            .checked_sub(ask.data[ORDER_SPENT_OFFSET])
            .ok_or_else(|| b"MathUnderflow")?;

        let assets_base_value = quote.data[QUOTE_PRICE_OFFSET]
            .checked_mul(itp_unburnt)
            .ok_or_else(|| b"MathOverflow")?;

        Ok(assets_base_value.to_u128())
    }

    pub fn total_supply(&self) -> Result<U128, Vec<u8>> {
        let vault = Self::_storage();

        let ret = self.static_call_ret(
            vault.gate_to_castle.get(),
            IFactor::getTotalOrderCall {
                index_id: vault.index_id.get().to(),
            },
        )?;

        let bid = Vector::from_vec(ret._0);
        let ask = Vector::from_vec(ret._1);

        let itp_available = bid.data[ORDER_REALIZED_OFFSET]
            .checked_sub(ask.data[ORDER_REMAIN_OFFSET])
            .ok_or_else(|| b"MathUnderflow")?;

        Ok(itp_available.to_u128())
    }

    pub fn get_total_assets(&self) -> Result<U128, Vec<u8>> {
        let vault = Self::_storage();

        let ret1 = self.static_call_ret(
            vault.gate_to_castle.get(),
            IFactor::getTotalOrderCall {
                index_id: vault.index_id.get().to(),
            },
        )?;

        let ret2 = self.static_call(
            vault.gate_to_castle.get(),
            IFactor::getIndexQuoteCall {
                index_id: vault.index_id.get().to(),
                vendor_id: vault.vendor_id.get().to(),
            },
        )?;

        let bid = Vector::from_vec(ret1._0);
        let ask = Vector::from_vec(ret1._1);
        let quote = Vector::from_vec(ret2);

        let itp_unburnt = bid.data[ORDER_REALIZED_OFFSET]
            .checked_sub(ask.data[ORDER_SPENT_OFFSET])
            .ok_or_else(|| b"MathUnderflow")?;

        let assets_base_value = quote.data[QUOTE_PRICE_OFFSET]
            .checked_mul(itp_unburnt)
            .ok_or_else(|| b"MathOverflow")?;

        Ok(assets_base_value.to_u128())
    }

    pub fn convert_to_assets(&self, amount: u128) -> Result<U128, Vec<u8>> {
        let vault = Self::_storage();

        let ret = self.static_call(
            vault.gate_to_castle.get(),
            IFactor::getIndexQuoteCall {
                index_id: vault.index_id.get().to(),
                vendor_id: vault.vendor_id.get().to(),
            },
        )?;

        let quote = Vector::from_vec(ret);

        let assets_base_value = quote.data[QUOTE_PRICE_OFFSET]
            .checked_mul(Amount::from_u128_raw(amount))
            .ok_or_else(|| b"MathOverflow")?;

        Ok(assets_base_value.to_u128())
    }

    pub fn convert_to_shares(&self, amount: u128) -> Result<U128, Vec<u8>> {
        let vault = Self::_storage();

        let ret = self.static_call(
            vault.gate_to_castle.get(),
            IFactor::getIndexQuoteCall {
                index_id: vault.index_id.get().to(),
                vendor_id: vault.vendor_id.get().to(),
            },
        )?;

        let quote = Vector::from_vec(ret);

        let assets_base_value = Amount::from_u128_raw(amount)
            .checked_div(quote.data[QUOTE_PRICE_OFFSET])
            .ok_or_else(|| b"MathOverflow")?;

        Ok(assets_base_value.to_u128())
    }

    pub fn transfer(&mut self, to: Address, value: U256) -> Result<(), Vec<u8>> {
        let vault = Self::_storage();

        self.external_call(
            vault.gate_to_castle.get(),
            IFactor::submitTransferOrderCall {
                index_id: vault.index_id.get().to(),
                receiver: to,
                amount: Amount::try_from_u256(value)
                    .ok_or_else(|| b"MathOverflow")?
                    .to_u128_raw(),
            },
        )?;

        Ok(())
    }

    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Vec<u8>> {
        todo!()
    }

    pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
        todo!()
    }

    pub fn approve(&mut self, spender: Address, value: U256) -> Result<bool, Vec<u8>> {
        todo!()
    }

    pub fn upgrade_to_and_call(
        &mut self,
        new_implementation: Address,
        data: Bytes,
    ) -> Result<(), Vec<u8>> {
        todo!()
    }
    
    pub fn set_version(&mut self) -> Result<(), Vec<u8>> {
        let mut vault = Self::_storage();
        if vault.version.get() > VERSION_NUMBER {
            Err(b"Version cannot be downgraded")?;
        }
        vault.version.set(VERSION_NUMBER);
        todo!()
    }
    
    pub fn get_version(&self) -> U32 {
        VERSION_NUMBER
    }
}
