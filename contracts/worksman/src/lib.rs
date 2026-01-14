// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{hex, uint, Address, U256};
use alloy_sol_types::{sol, SolCall};
use common_contracts::{
    contracts::{calls::InnerCall, keep::Keep, storage::StorageSlot},
    interfaces::vault::IVault,
};
use stylus_sdk::{keccak_const, prelude::*, storage::StorageAddress};

pub const WORKSMAN_STORAGE_SLOT: U256 = {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"Worksman.STORAGE_SLOT")
        .finalize();
    U256::from_be_bytes(HASH).wrapping_sub(uint!(1_U256))
};

const CREATION_CODE: &[u8] = include_bytes!("../code.txt");

sol! {
    interface IGate {
        function initialize(address implementation, bytes calldata data) external;

        function implementation() external view returns (address);
    }
}

#[storage]
struct WorksmanStorage {
    prototype: StorageAddress,
}

#[storage]
#[entrypoint]
pub struct Worksman;

impl Worksman {
    fn _storage() -> WorksmanStorage {
        StorageSlot::get_slot::<WorksmanStorage>(WORKSMAN_STORAGE_SLOT)
    }

    fn _build_gate(&mut self, prototype: Address) -> Result<Address, Vec<u8>> {
        let creation_code = hex::decode(CREATION_CODE).map_err(|_| b"Failed to decode hex")?;
        let gate = unsafe {
            self.vm()
                .deploy(&creation_code, U256::ZERO, None, deploy::CachePolicy::Flush)?
        };

        let castle = self.top_level();

        let IGate::implementationReturn { _0: implementation } = self
            .static_call_ret(prototype, IGate::implementationCall {})
            .map_err(|_| "Failed to obtain implementation")?;

        let IVault::vaultImplementationReturn {
            _0: vault_implementation,
        } = self
            .static_call_ret(prototype, IVault::vaultImplementationCall {})
            .map_err(|_| "Failed to obtain vault implementation")?;

        let init_vault = IVault::initializeCall {
            owner: prototype,
            vault_implementation,
            gate_to_castle: castle,
        };

        let init_gate = IGate::initializeCall {
            implementation,
            data: init_vault.abi_encode().into(),
        };

        self.external_call(gate, init_gate)
            .map_err(|_| b"Failed to initialize gate")?;

        let clone_implementation = IVault::cloneImplementationCall {
            to: gate,
            new_owner: castle,
        };

        self.external_call(prototype, clone_implementation)
            .map_err(|_| b"Failed to clone implementation")?;

        Ok(gate)
    }
}

#[public]
impl Worksman {
    pub fn set_vault_prototype(&mut self, vault: Address) -> Result<(), Vec<u8>> {
        let keep = Keep::storage();
        if keep.worksman.get().is_zero() {
            Err(b"Worksman not appointed")?;
        }

        let mut storage = Self::_storage();
        storage.prototype.set(vault);
        Ok(())
    }

    pub fn build_vault(&mut self) -> Result<Address, Vec<u8>> {
        let keep = Keep::storage();
        if keep.worksman.get().is_zero() {
            Err(b"Worksman not appointed")?;
        }

        let storage = Self::_storage();
        let prototype = storage.prototype.get();

        let vault = self._build_gate(prototype)?;
        Ok(vault)
    }
}
