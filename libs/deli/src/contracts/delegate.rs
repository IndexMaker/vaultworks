use alloc::{vec, vec::Vec};

use alloy_primitives::{Address, B256, U8};

use stylus_sdk::{
    prelude::*,
    storage::{StorageAddress, StorageB256, StorageU8},
};

pub const ACCESS_MODE_NONE: u8 = 0;
pub const ACCESS_MODE_PROTECTED: u8 = 1;

#[storage]
pub struct Delegate {
    contract_address: StorageAddress,
    required_role: StorageB256,
    access_mode: StorageU8,
}

impl Delegate {
    pub fn initialize(&mut self, contract_address: Address, required_role: Option<B256>) {
        self.contract_address.set(contract_address);
        if let Some(required_role) = required_role {
            self.access_mode.set(U8::from(ACCESS_MODE_PROTECTED));
            self.required_role.set(required_role);
        } else {
            self.access_mode.set(U8::from(ACCESS_MODE_NONE));
            self.required_role.erase();
        }
    }

    pub fn erase(&mut self) {
        self.contract_address.erase();
        self.access_mode.erase();
        self.required_role.erase();
    }

    pub fn get_contract_address(&self) -> Option<Address> {
        let contract_address = self.contract_address.get();
        if contract_address.is_zero() {
            None
        } else {
            Some(contract_address)
        }
    }

    pub fn get_required_role(&self) -> Result<Option<B256>, Vec<u8>> {
        let mode = self.access_mode.get();
        match mode.to::<u8>() {
            ACCESS_MODE_PROTECTED => {
                let required_role = self.required_role.get();
                Ok(Some(required_role))
            }
            ACCESS_MODE_NONE => Ok(None),
            _ => Err(b"Invalid access mode")?,
        }
    }
}
