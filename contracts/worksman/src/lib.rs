// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{uint, Address, U256};
use common_contracts::{
    contracts::{calls::InnerCall, keep::Keep, storage::StorageSlot},
    interfaces::vault::IVault,
};
use stylus_sdk::{
    keccak_const,
    prelude::*,
    storage::{StorageAddress, StorageBool, StorageMap, StorageVec},
};

pub const WORKSMAN_STORAGE_SLOT: U256 = {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"Keep.STORAGE_SLOT")
        .finalize();
    U256::from_be_bytes(HASH).wrapping_sub(uint!(1_U256))
};

#[storage]
struct WorksmanStorage {
    all_vaults: StorageMap<Address, StorageBool>,
    free_vaults: StorageVec<StorageAddress>,
}

impl WorksmanStorage {
    fn next_vault(&mut self) -> Result<Address, Vec<u8>> {
        let last_index = self.free_vaults.len();
        if let Some(vault) = self.free_vaults.get(last_index - 1) {
            self.free_vaults.erase_last();
            Ok(vault)
        } else {
            Err(b"No more Vaults available")?
        }
    }
}

#[storage]
#[entrypoint]
pub struct Worksman;

impl Worksman {
    fn _storage() -> WorksmanStorage {
        StorageSlot::get_slot::<WorksmanStorage>(WORKSMAN_STORAGE_SLOT)
    }
}

#[public]
impl Worksman {
    pub fn add_vault(&mut self, vault: Address) -> Result<(), Vec<u8>> {
        let mut storage = Self::_storage();
        let mut vault_setter = storage.all_vaults.setter(vault);
        if vault_setter.get() {
            Err(b"Vault already added")?;
        }
        let IVault::ownerReturn { _0: owner } =
            self.static_call_ret(vault, IVault::ownerCall {})?;

        if owner != self.top_level() {
            Err(b"Vault ownership must be returned")?;
        }
        vault_setter.set(true);
        storage.free_vaults.push(vault);
        Ok(())
    }

    pub fn build_vault(&mut self) -> Result<Address, Vec<u8>> {
        let keep = Keep::storage();
        if keep.worksman.get().is_zero() {
            Err(b"Worksman not appointed")?;
        }
        let mut storage = Self::_storage();
        let vault = storage.next_vault()?;

        Ok(vault)
    }
}
