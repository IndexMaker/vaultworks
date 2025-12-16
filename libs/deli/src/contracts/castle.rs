use alloc::{vec, vec::Vec};

use alloy_primitives::{aliases::B32, uint, Address, B256, U256};

use stylus_sdk::{
    keccak_const,
    prelude::*,
    storage::{StorageAddress, StorageGuard, StorageMap},
};

use crate::{contracts::acl::Role, log_msg, storage::StorageSlot};

use super::{acl::AccessControlList, delegate::Delegate};

pub const CASTLE_ADMIN_ROLE: [u8; 32] = keccak_const::Keccak256::new()
    .update(b"Castle.ADMIN_ROLE")
    .finalize();

pub const CASTLE_STORAGE_SLOT: U256 = {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"Castle.STORAGE_SLOT")
        .finalize();
    U256::from_be_bytes(HASH).wrapping_sub(uint!(1_U256))
};

#[storage]
pub struct CastleStorage {
    delegates: StorageMap<B32, Delegate>,
    acl: AccessControlList,
    castle: StorageAddress,
}

impl CastleStorage {
    pub fn storage() -> CastleStorage {
        StorageSlot::get_slot::<CastleStorage>(CASTLE_STORAGE_SLOT)
    }

    pub fn construct(&mut self, castle: Address) -> Result<(), Vec<u8>> {
        if self.has_castle() {
            Err(b"Castle already constructed")?;
        }
        self.castle.set(castle);
        Ok(())
    }

    pub fn has_castle(&self) -> bool { 
        !self.castle.get().is_zero()
    }

    pub fn get_castle(&self) -> Address {
        self.castle.get()
    }

    pub fn get_function_delegate_from_calldata(
        &self,
        calldata: &[u8],
    ) -> Result<Option<(Address, Option<StorageGuard<'_, Role>>)>, Vec<u8>> {
        let fun_sel = B32::from_slice(calldata.get(0..4).ok_or_else(|| b"Calldata invalid")?);
        self.get_function_delegate(fun_sel)
    }

    pub fn get_function_delegate(
        &self,
        fun_sel: B32,
    ) -> Result<Option<(Address, Option<StorageGuard<'_, Role>>)>, Vec<u8>> {
        let delegate = self.delegates.get(fun_sel);
        if let Some(contract_address) = delegate.get_contract_address() {
            if let Some(required_role) = delegate.get_required_role()? {
                let role = self.acl.get_role(required_role);
                Ok(Some((contract_address, Some(role))))
            } else {
                Ok(Some((contract_address, None)))
            }
        } else {
            Ok(None)
        }
    }

    pub fn get_function_delegate_address(&self, fun_sel: B32) -> Option<Address> {
        let delegate = self.delegates.get(fun_sel);
        if let Some(contract_address) = delegate.get_contract_address() {
            Some(contract_address)
        } else {
            None
        }
    }

    pub fn get_acl(&self) -> &AccessControlList {
        &self.acl
    }

    pub fn get_acl_mut(&mut self) -> &mut AccessControlList {
        &mut self.acl
    }

    pub fn set_functions(
        &mut self,
        contract_address: Option<Address>,
        required_role: Option<B256>,
        fun_selectors: &Vec<B32>,
    ) {
        for fun_sel in fun_selectors {
            let mut delegate = self.delegates.setter(*fun_sel);
            if let Some(contract_address) = contract_address {
                log_msg!(
                    "Assigning function {} delegation to {:?} (previously assigned to {:?})",
                    fun_sel,
                    contract_address,
                    delegate.get_contract_address()
                );
                delegate.initialize(contract_address, required_role);
            } else {
                log_msg!(
                    "Removing function {} delegation (previously assigned to {:?})",
                    fun_sel,
                    delegate.get_contract_address()
                );
                delegate.erase();
            }
        }
    }
}
