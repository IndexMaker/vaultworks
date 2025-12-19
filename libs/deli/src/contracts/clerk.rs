use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, U128, U256, uint};
use stylus_sdk::{
    keccak_const, prelude::*, storage::{StorageAddress, StorageBool, StorageBytes, StorageMap}
};

use crate::contracts::storage::StorageSlot;

#[storage]
pub struct ClerkStorage {
    vectors: StorageMap<U128, StorageBytes>,
    presence: StorageMap<U128, StorageBool>,
    abacus: StorageAddress,
    owner: StorageAddress,
}

pub const CLERK_STORAGE_SLOT: U256 = {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"Clerk.STORAGE_SLOT")
        .finalize();
    U256::from_be_bytes(HASH).wrapping_sub(uint!(1_U256))
};

impl ClerkStorage {
    pub fn storage() -> ClerkStorage {
        StorageSlot::get_slot::<ClerkStorage>(CLERK_STORAGE_SLOT)
    }

    pub fn initialize(&mut self, owner: Address, abacus: Address) -> Result<(), Vec<u8>> {
        if !self.owner.get().is_zero() {
            Err(b"Clerk already has an owner")?;
        }
        self.owner.set(owner);
        self.abacus.set(abacus);
        Ok(())
    }

    pub fn is_owner(&self, attendee: Address) -> bool {
        self.owner.get() == attendee
    }

    pub fn only_owner(&self, attendee: Address) -> Result<(), Vec<u8>> {
        if !self.is_owner(attendee) {
            Err(b"Unauthorised access")?;
        }
        Ok(())
    }

    pub fn get_abacus_address(&self) -> Address {
        self.abacus.get()
    }

    pub fn store_bytes(&mut self, id: U128, data: Vec<u8>) {
        let mut vector = self.vectors.setter(id);
        vector.set_bytes(data);
        self.presence.setter(id).set(true);
    }

    pub fn fetch_bytes(&self, id: U128) -> Option<Vec<u8>> {
        if !self.presence.get(id) {
            return None;
        }
        let vector = self.vectors.getter(id);
        Some(vector.get_bytes())
    }
}
