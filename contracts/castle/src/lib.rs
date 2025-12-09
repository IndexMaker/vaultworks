// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{aliases::B32, Address, B256, U256, U8};

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
    storage::{StorageAddress, StorageB256, StorageMap, StorageU8},
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
    hash: StorageB256,
    mode: StorageU8,
}

// Role is:
// * protected ==> ACCESS_MODE_INCLUDE : because we need to check if user is included in the role
// * public ==> ACCESS_MODE_NONE : because we will not do any checks
// * unset ==> ACCESS_MODE_NONE : because we will not do any checks (skipped by implementation == ZERO)
// Note: Exclude makes no sense as if your address is excluded, you can always use different one.
pub const ACCESS_MODE_NONE: u8 = 0;
pub const ACCESS_MODE_INCLUDE: u8 = 1;

pub const SET_ROLES_ROLE: [u8; 32] = keccak_const::Keccak256::new()
    .update(b"Castle.SET_ROLES_ROLE")
    .finalize();

/// One to Many proxy (aka Diamond)
///
/// All calls go through access control check.
/// 
/// Note: This is lightweight variant of ERC2535.
/// 
/// * ADMIN ==> assigns roles to the users,
/// * SET_ROLES_ROLE ==> assigns implementations to the roles.
///
/// We don't implement complex loupe interface, and we provide three methods to
/// managing available roles:
/// 
/// * `setProtectedRoles(address implementation, bytes4[] roleIds)` ==> these roles must have users assigned to them
/// * `setPublicRoles(address implementation, bytes4[] roleIds)` ==> these roles are publicly available
/// * `unsetRoles(bytes4[] roleIds)` ==> roles can be removed
/// 
/// Additionally we support only listing of the implementations assigned to
/// specific roles:
/// 
/// * `getRoleAssingees(bytes4[] roleIds) view (address[])` ==> list implementations for the specific roles
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
        mode: U8,
    ) -> Result<(), Vec<u8>> {
        self.access.only_role(SET_ROLES_ROLE.into())?;
        if implementation == self.vm().contract_address() {
            Err(b"Expected implementation address")?;
        }
        for role_id in role_ids {
            let hash = self.vm().native_keccak256(role_id.as_slice());
            let mut role = self.roles.setter(*role_id);
            log_msg!(
                "Replacing implementation: {} => {} for role: {} ({} {})",
                role.implementation.get(),
                address,
                role_id,
                hash,
                mode
            );
            role.implementation.set(implementation);
            role.hash.set(hash);
            role.mode.set(mode);
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
        self.access_enumerable._grant_role(
            SET_ROLES_ROLE.into(),
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
    /// Only users added to the role will be able to access listed functions.
    ///
    pub fn set_protected_roles(
        &mut self,
        implementation: Address,
        role_ids: Vec<B32>,
    ) -> Result<(), Vec<u8>> {
        self._set_roles(implementation, &role_ids, U8::from(ACCESS_MODE_INCLUDE))?;
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
    /// Everyone will be able to access listed functions.
    ///
    pub fn set_public_roles(
        &mut self,
        implementation: Address,
        role_ids: Vec<B32>,
    ) -> Result<(), Vec<u8>> {
        self._set_roles(implementation, &role_ids, U8::from(ACCESS_MODE_NONE))?;
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
        self._set_roles(Address::ZERO, &role_ids, U8::from(ACCESS_MODE_NONE))?;
        self.vm()
            .emit_log(&CastleRolesUnset { roles: role_ids }.encode_data(), 1);
        Ok(())
    }

    /// Obtain list of implementations assigned to specific roles.
    /// 
    /// Parameters
    /// ----------
    /// - role_ids: A list of function selectors (first 4 bytes of EVM ABI call encoding)
    /// 
    pub fn get_role_assignees(&self, role_ids: Vec<B32>) -> Result<Vec<Address>, Vec<u8>> {
        let mut roles = Vec::new();
        for role_id in &role_ids {
            let role = self.roles.get(*role_id);
            roles.push(role.implementation.get());
        }
        Ok(roles)
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

        let mode = role.mode.get();
        match mode.to::<u8>() {
            ACCESS_MODE_INCLUDE => {
                // Role is protected, only select addresses can access.
                let hash = role.hash.get();
                self.access.only_role(hash)?
            }
            ACCESS_MODE_NONE => {
                // Role is public.
            }
            _ => Err(b"Invalid access mode")?,
        };

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
