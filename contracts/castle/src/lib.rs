// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{aliases::B32, Address, B256, U256};

use alloy_sol_types::{sol, SolEvent};
use deli::log_msg;
use openzeppelin_stylus::{
    access::control::{
        self,
        extensions::{AccessControlEnumerable, IAccessControlEnumerable},
        AccessControl, IAccessControl,
    },
    utils::introspection::erc165::IErc165,
};
use stylus_sdk::{
    keccak_const,
    prelude::*,
    storage::{StorageAddress, StorageB256, StorageMap},
    ArbResult,
};

sol! {
    event CastleProtectedRolesSet(address _address, bytes4[] roles);
    event CastlePublicRolesSet(address _address, bytes4[] roles);
    event CastleRolesUnset(bytes4[] roles);
}

#[derive(SolidityError, Debug)]
enum Error {
    UnauthorizedAccount(control::AccessControlUnauthorizedAccount),
    BadConfirmation(control::AccessControlBadConfirmation),
}

impl From<control::Error> for Error {
    fn from(value: control::Error) -> Self {
        match value {
            control::Error::UnauthorizedAccount(e) => Error::UnauthorizedAccount(e),
            control::Error::BadConfirmation(e) => Error::BadConfirmation(e),
        }
    }
}

#[storage]
struct Role {
    implementation: StorageAddress,
    hash_or_zero: StorageB256,
}

pub const SET_ROLES_ROLE: [u8; 32] = keccak_const::Keccak256::new()
    .update(b"Castle.SET_ROLES_ROLE")
    .finalize();

/// One to Many proxy (aka Diamond)
///
/// All calls go through access control check.
///
#[entrypoint]
#[storage]
struct Castle {
    access: AccessControl,
    access_enumerable: AccessControlEnumerable,
    roles: StorageMap<B32, Role>,
}

impl Castle {
    fn _set_roles(
        &mut self,
        implementation: Address,
        role_ids: &Vec<B32>,
        with_access_control: bool,
    ) -> Result<(), Vec<u8>> {
        self.access.only_role(SET_ROLES_ROLE.into())?;
        for role_id in role_ids {
            let hash_or_zero = if with_access_control {
                self.vm().native_keccak256(role_id.as_slice())
            } else {
                B256::ZERO
            };
            let mut role = self.roles.setter(*role_id);
            log_msg!(
                "Replacing implementation: {} => {} for role: {} ({})",
                role.implementation.get(),
                address,
                role_id,
                hash
            );
            role.implementation.set(implementation);
            role.hash_or_zero.set(hash_or_zero);
        }
        Ok(())
    }
}

#[public]
#[implements(IAccessControl<Error = control::Error>, IAccessControlEnumerable<Error = control::extensions::enumerable::Error>, IErc165)]
impl Castle {
    #[constructor]
    pub fn constructor(&mut self, admin: Address) -> Result<(), Vec<u8>> {
        self.access_enumerable._grant_role(
            AccessControl::DEFAULT_ADMIN_ROLE.into(),
            admin,
            &mut self.access,
        );
        log_msg!("Set initial owner to: {}", initial_owner);
        Ok(())
    }

    /// Associate function selectors with implementation address
    /// adding access control based on hash of the selector (**protected**).
    ///
    /// Parameters
    /// ----------
    /// - implementation: An address of the contract implementing the functions
    /// - role_ids: A list of function selectors (first 4 bytes of EVM ABI call encoding)
    ///
    pub fn set_protected_roles(
        &mut self,
        implementation: Address,
        role_ids: Vec<B32>,
    ) -> Result<(), Vec<u8>> {
        self._set_roles(implementation, &role_ids, true)?;
        self.vm().emit_log(
            &CastleProtectedRolesSet {
                _address: implementation,
                roles: role_ids,
            }
            .encode_data(),
            1,
        );
        Ok(())
    }

    /// Associate function selectors with implementation address
    /// adding access control based on hash of the selector (**public**).
    ///
    /// Parameters
    /// ----------
    /// - implementation: An address of the contract implementing the functions
    /// - role_ids: A list of function selectors (first 4 bytes of EVM ABI call encoding)
    ///
    pub fn set_public_roles(
        &mut self,
        implementation: Address,
        role_ids: Vec<B32>,
    ) -> Result<(), Vec<u8>> {
        self._set_roles(implementation, &role_ids, false)?;
        self.vm().emit_log(
            &CastlePublicRolesSet {
                _address: implementation,
                roles: role_ids,
            }
            .encode_data(),
            1,
        );
        Ok(())
    }

    /// Disassociate function selectors with implementation address.
    ///
    /// Parameters
    /// ----------
    /// - role_ids: A list of function selectors (first 4 bytes of EVM ABI call encoding)
    ///
    pub fn unset_roles(&mut self, role_ids: Vec<B32>) -> Result<(), Vec<u8>> {
        self._set_roles(Address::ZERO, &role_ids, false)?;
        self.vm()
            .emit_log(&CastleRolesUnset { roles: role_ids }.encode_data(), 1);
        Ok(())
    }

    #[payable]
    #[fallback]
    fn fallback(&mut self, calldata: &[u8]) -> ArbResult {
        let role_id = B32::from_slice(calldata.get(0..4).ok_or_else(|| b"Calldata invalid")?);
        let role = self.roles.get(role_id);

        let implementation = role.implementation.get();
        if implementation == Address::ZERO {
            log_msg!("Address not found for role: {}", role_id);
            Err(b"Role not found")?;
        }

        let hash_or_zero = role.hash_or_zero.get();
        if !hash_or_zero.is_zero() {
            self.access.only_role(hash_or_zero)?;
        }

        log_msg!("Delegate to: {} for role: {}", role_address, role_id);
        unsafe { Ok(self.vm().delegate_call(&self, implementation, calldata)?) }
    }
}

#[public]
impl IAccessControl for Castle {
    type Error = control::Error;

    fn has_role(&self, role: B256, account: Address) -> bool {
        self.access.has_role(role, account)
    }

    fn only_role(&self, role: B256) -> Result<(), Self::Error> {
        self.access.only_role(role)
    }

    fn get_role_admin(&self, role: B256) -> B256 {
        self.access.get_role_admin(role)
    }

    fn grant_role(&mut self, role: B256, account: Address) -> Result<(), Self::Error> {
        let admin_role = self.get_role_admin(role);
        self.only_role(admin_role)?;
        self.access_enumerable
            ._grant_role(role, account, &mut self.access);
        Ok(())
    }

    fn revoke_role(&mut self, role: B256, account: Address) -> Result<(), Self::Error> {
        let admin_role = self.get_role_admin(role);
        self.only_role(admin_role)?;
        self.access_enumerable
            ._revoke_role(role, account, &mut self.access);
        Ok(())
    }

    fn renounce_role(&mut self, role: B256, confirmation: Address) -> Result<(), Self::Error> {
        if self.vm().msg_sender() != confirmation {
            return Err(control::Error::BadConfirmation(
                control::AccessControlBadConfirmation {},
            ));
        }

        self.access_enumerable
            ._revoke_role(role, confirmation, &mut self.access);
        Ok(())
    }
}

#[public]
impl IAccessControlEnumerable for Castle {
    type Error = control::extensions::enumerable::Error;

    fn get_role_member(&self, role: B256, index: U256) -> Result<Address, Self::Error> {
        self.access_enumerable.get_role_member(role, index)
    }

    fn get_role_member_count(&self, role: B256) -> U256 {
        self.access_enumerable.get_role_member_count(role)
    }
}

#[public]
impl IErc165 for Castle {
    fn supports_interface(&self, interface_id: B32) -> bool {
        self.access.supports_interface(interface_id)
            || self.access_enumerable.supports_interface(interface_id)
    }
}
