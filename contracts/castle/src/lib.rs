// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{aliases::B32, Address};

use alloy_sol_types::{sol, SolEvent};
use deli::log_msg;
use openzeppelin_stylus::access::ownable::{IOwnable, Ownable};
use stylus_sdk::{
    prelude::*,
    storage::{StorageAddress, StorageMap},
    ArbResult,
};

sol! {
    event CastleRoles(address _address, bytes4[] roles);
}

#[entrypoint]
#[storage]
struct Castle {
    ownable: Ownable,
    roles: StorageMap<B32, StorageAddress>,
}

#[public]
#[implements(IOwnable)]
impl Castle {
    #[constructor]
    pub fn constructor(&mut self, initial_owner: Address) -> Result<(), Vec<u8>> {
        self.ownable.constructor(initial_owner)?;
        log_msg!("Set initial owner to: {}", initial_owner);
        Ok(())
    }

    pub fn set_roles(&mut self, address: Address, role_ids: Vec<B32>) -> Result<(), Vec<u8>> {
        self.ownable.only_owner()?;
        for role_id in &role_ids {
            let _old_address = self.roles.replace(role_id.clone(), address);
            log_msg!(
                "Replacing: {} => {} for role: {}",
                _old_address,
                address,
                role_id
            );
        }
        self.vm().emit_log(
            &CastleRoles {
                _address: address,
                roles: role_ids,
            }
            .encode_data(),
            1,
        );
        Ok(())
    }

    #[payable]
    #[fallback]
    fn fallback(&mut self, calldata: &[u8]) -> ArbResult {
        let role_id = B32::from_slice(calldata.get(0..4).ok_or_else(|| b"Calldata invalid")?);
        let role_address = self.roles.get(role_id);

        if role_address == Address::ZERO {
            log_msg!("Address not found for role: {}", role_id);
            Err(b"Role not found")?;
        }

        log_msg!("Delegate to: {} for role: {}", role_address, role_id);
        unsafe { Ok(self.vm().delegate_call(&self, role_address, calldata)?) }
    }
}

#[public]
impl IOwnable for Castle {
    fn owner(&self) -> Address {
        let owner = self.ownable.owner();
        log_msg!("Current owner is: {}", owner);
        owner
    }

    fn transfer_ownership(&mut self, new_owner: Address) -> Result<(), Vec<u8>> {
        log_msg!("Transfer ownership to: {}", new_owner);
        Ok(self.ownable.transfer_ownership(new_owner)?)
    }

    fn renounce_ownership(&mut self) -> Result<(), Vec<u8>> {
        log_msg!("Renounce ownership");
        Ok(self.ownable.renounce_ownership()?)
    }
}
