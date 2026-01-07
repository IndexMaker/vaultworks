use alloc::{vec, vec::Vec};

use alloy_primitives::{uint, U128, U256};
use common::vector::Vector;
use stylus_sdk::{
    keccak_const,
    prelude::*,
    storage::{StorageBool, StorageBytes, StorageMap, StorageU128},
};

use crate::contracts::storage::StorageSlot;

pub const SCRATCH_1: U128 = uint!(1_U128);
pub const SCRATCH_2: U128 = uint!(2_U128);
pub const SCRATCH_3: U128 = uint!(3_U128);
pub const SCRATCH_4: U128 = uint!(4_U128);

pub const FIRST_DYNAMIC_ID: U128 = uint!(100_U128);

#[storage]
pub struct ClerkStorage {
    vectors: StorageMap<U128, StorageBytes>,
    presence: StorageMap<U128, StorageBool>,
    last_vector: StorageU128,
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

    pub fn constructor(&mut self) -> Result<(), Vec<u8>> {
        if self.is_constructed() {
            Err(b"Clerk storage already constructed")?;
        }
        self.last_vector.set(FIRST_DYNAMIC_ID);
        Ok(())
    }

    pub fn is_constructed(&self) -> bool {
        !self.last_vector.get().is_zero()
    }

    pub fn next_vector(&mut self) -> U128 {
        let value = self.last_vector.get() + U128::ONE;
        self.last_vector.set(value);
        value
    }

    pub fn len_bytes(&self, id: U128) -> usize {
        self.vectors.get(id).len()
    }

    pub fn len_vector(&self, id: U128) -> usize {
        self.len_bytes(id) / size_of::<u128>()
    }

    pub fn store_bytes(&mut self, id: U128, data: impl AsRef<[u8]>) {
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

    pub fn store_vector(&mut self, vector_id: U128, vector: Vector) {
        self.store_bytes(vector_id, vector.to_vec());
    }

    pub fn fetch_vector(&self, vector_id: U128) -> Option<Vector> {
        let data = self.fetch_bytes(vector_id)?;
        Some(Vector::from_vec(data))
    }

}
