use alloc::{string::String, vec::Vec};
use alloy_primitives::{uint, Address, B256, U256};
use stylus_sdk::{
    abi::Bytes,
    keccak_const,
    prelude::*,
    storage::{StorageAddress, StorageBool},
};

use crate::contracts::storage::StorageSlot;

pub const UPGRADE_INTERFACE_VERSION: &str = "5.0.0";

pub const IMPLEMENTATION_SLOT: B256 = {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"eip1967.proxy.implementation")
        .finalize();
    B256::new(
        U256::from_be_bytes(HASH)
            .wrapping_sub(uint!(1_U256))
            .to_be_bytes(),
    )
};

pub const LOGIC_FLAG_SLOT: B256 = {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"Stylus.uups.logic.flag")
        .finalize();
    let slot = U256::from_be_bytes(HASH)
        .wrapping_sub(uint!(1_U256))
        .to_be_bytes::<32>();

    B256::new(slot)
};

pub struct Gate;

impl Gate {
    pub fn implementation() -> StorageAddress {
        StorageSlot::get_slot::<StorageAddress>(IMPLEMENTATION_SLOT)
    }

    pub fn logic_flag() -> StorageBool {
        StorageSlot::get_slot::<StorageBool>(LOGIC_FLAG_SLOT)
    }

    pub fn construct_logic() {
        Self::logic_flag().set(true);
    }

    pub fn is_logic() -> bool {
        Self::logic_flag().get()
    }

    pub fn only_proxy() -> Result<(), Vec<u8>> {
        if Self::is_logic() || Self::implementation().get() == Address::ZERO {
            Err(b"UUPSUnauthorizedCallContext".into())
        } else {
            Ok(())
        }
    }

    pub fn only_delegated() -> Result<(), Vec<u8>> {
        if !Self::is_logic() {
            Ok(())
        } else {
            Err(b"UUPSUnauthorizedCallContext".into())
        }
    }

    pub fn upgrade_interface_version() -> String {
        UPGRADE_INTERFACE_VERSION.into()
    }

    pub fn upgrade_to_and_call(
        host_access: &mut (impl HostAccess + TopLevelStorage),
        new_implementation: Address,
        data: Bytes,
    ) -> Result<(), Vec<u8>> {
        Self::only_proxy()?;
        Gate::implementation().set(new_implementation);
        unsafe {
            host_access
                .vm()
                .delegate_call(&host_access, new_implementation, data.as_slice())
        }?;
        Ok(())
    }

    pub fn proxiable_uuid(&self) -> Result<B256, Vec<u8>> {
        Ok(IMPLEMENTATION_SLOT)
    }
}
