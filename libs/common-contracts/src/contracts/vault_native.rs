use alloc::{vec, vec::Vec};

use alloy_primitives::{uint, Address, U256};
use common::{amount::Amount, vector::Vector};
use stylus_sdk::{
    keccak_const,
    prelude::*,
    storage::{StorageAddress, StorageBool, StorageMap, StorageU128},
};

use crate::{
    contracts::{calls::InnerCall, formulas::Quote, storage::StorageSlot, vault::VaultStorage},
    interfaces::{guildmaster::IGuildmaster, steward::ISteward},
};

pub const VAULT_NATIVE_STORAGE_SLOT: U256 = {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"VaultNative.STORAGE_SLOT")
        .finalize();
    U256::from_be_bytes(HASH).wrapping_sub(uint!(1_U256))
};

#[storage]
pub struct TraderOrder {
    pub withdraw_ready: StorageU128,
}

#[storage]
pub struct VaultNativeStorage {
    pub vendor_id: StorageU128,
    pub custody: StorageAddress,
    pub collateral_asset: StorageAddress,
    pub max_order_size: StorageU128,
    pub trader_orders: StorageMap<Address, TraderOrder>,
    pub operators: StorageMap<Address, StorageBool>,
}

impl VaultNativeStorage {
    pub fn storage() -> VaultNativeStorage {
        StorageSlot::get_slot::<VaultNativeStorage>(VAULT_NATIVE_STORAGE_SLOT)
    }

    pub fn set_operator(&mut self, operator: Address, status: bool) {
        self.operators.setter(operator).set(status);
    }

    pub fn is_operator(&self, operator: Address) -> bool {
        self.operators.get(operator)
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

    pub fn update_quote(
        &self,
        vault: &VaultStorage,
        caller: &mut impl InnerCall,
    ) -> Result<(), Vec<u8>> {
        caller.external_call(
            vault.gate_to_castle.get(),
            IGuildmaster::updateIndexQuoteCall {
                vendor_id: self.vendor_id.get().to(),
                index_id: vault.index_id.get().to(),
            },
        )?;
        Ok(())
    }

    pub fn get_asset_contribution_fractions(
        &self,
        vault: &VaultStorage,
        caller: &impl InnerCall,
    ) -> Result<Vector, Vec<u8>> {
        // Not the most efficient way of getting unit vector of same length...
        let ISteward::getIndexAssetsCountReturn { _0: count } = caller.static_call_ret(
            vault.gate_to_castle.get(),
            ISteward::getIndexAssetsCountCall {
                index_id: vault.index_id.get().to(),
            },
        )?;

        let mut unit_vector = Vector { data: vec![] };
        unit_vector.data.resize(count as usize, Amount::ONE);
        Ok(unit_vector)
    }
}
