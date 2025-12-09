// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{aliases::B32, Address, B256, U8};

use alloy_sol_types::{SolCall, SolEvent};
use deli::{contracts::ICastle, log_msg};
use stylus_sdk::{
    keccak_const,
    prelude::*,
    storage::{StorageAddress, StorageB256, StorageMap, StorageU8, StorageVec},
    ArbResult,
};

pub const CASTLE_ADMIN_ROLE: [u8; 32] = keccak_const::Keccak256::new()
    .update(b"Castle.ADMIN_ROLE")
    .finalize();

pub const ACCESS_MODE_NONE: u8 = 0;
pub const ACCESS_MODE_PROTECTED: u8 = 1;

/// Lightweight Access Control List (ACL)
///
/// This is constructed from two mappings:
/// * `role => [assignee]` - a set of addresses assigned to each role
/// * `assignee => [role]` - a set of roles assigned to each address
///
/// We can assign to a role multiple assignees, or to an assignee multiple roles.
/// Either way we always update both mappings to keep integrity.
///
#[storage]
struct AccessControlList {
    roles: StorageMap<B256, StorageVec<StorageAddress>>,
    assignees: StorageMap<Address, StorageVec<StorageB256>>,
}

impl AccessControlList {
    fn _set_role(&mut self, attendee: Address, role: B256) -> Result<(), Vec<u8>> {
        let mut role_assignees = self.roles.setter(role);
        for index in 0..role_assignees.len() {
            let assignee = role_assignees.get(index).unwrap();
            if assignee.eq(&attendee) {
                Err(b"Role already assigned")?;
            }
        }
        role_assignees.push(attendee);
        self.assignees.setter(attendee).push(role);
        Ok(())
    }

    fn _remove_assignee_from_role(&mut self, role: B256, attendee: Address) -> Result<(), Vec<u8>> {
        let mut role_assignees = self.roles.setter(role);
        let assignee_count = role_assignees.len();
        if 0 == assignee_count {
            Err(b"Role does not exist")?;
        }
        let last_assignee = role_assignees.get(assignee_count - 1).unwrap();
        for index in 0..assignee_count {
            let mut assignee_setter = role_assignees.setter(index).unwrap();
            if assignee_setter.get().eq(&attendee) {
                if index < assignee_count - 1 {
                    assignee_setter.set(last_assignee);
                }
                role_assignees.erase_last();
                break;
            }
        }
        Ok(())
    }

    fn _remove_role_from_assignee(&mut self, attendee: Address, role: B256) -> Result<(), Vec<u8>> {
        let mut assignee_roles = self.assignees.setter(attendee);
        let role_count = assignee_roles.len();
        if 0 == role_count {
            Err(b"Attendee has no roles assigned")?;
        }
        let last_role = assignee_roles.get(role_count - 1).unwrap();
        for index in 0..role_count {
            let mut role_setter = assignee_roles.setter(index).unwrap();
            if role_setter.get().eq(&role) {
                if index < role_count - 1 {
                    role_setter.set(last_role);
                }
                assignee_roles.erase_last();
            }
        }
        Ok(())
    }

    fn _get_role_assignees(&self, role: B256) -> Result<Vec<Address>, Vec<u8>> {
        let role_assignees = self.roles.getter(role);
        let assignee_count = role_assignees.len();
        if 0 == assignee_count {
            Err(b"Role does not exist")?;
        }
        let mut assignees = Vec::with_capacity(assignee_count);
        for index in 0..assignee_count {
            let role_assignee = role_assignees.get(index).unwrap();
            assignees.push(role_assignee);
        }
        Ok(assignees)
    }

    fn _get_assigned_roles(&self, attendee: Address) -> Result<Vec<B256>, Vec<u8>> {
        let assigned_roles = self.assignees.getter(attendee);
        let role_count = assigned_roles.len();
        let mut roles = Vec::with_capacity(role_count);
        for index in 0..role_count {
            let role = assigned_roles.get(index).unwrap();
            roles.push(role);
        }
        Ok(roles)
    }

    fn _has_role(&self, role: &[u8; 32], attendee: Address) -> bool {
        let assigned_roles = self.assignees.get(attendee);
        for index in 0..assigned_roles.len() {
            let assigned_role = assigned_roles.get(index).unwrap();
            if assigned_role.eq(role) {
                return true;
            }
        }
        false
    }

    fn _unset_role(&mut self, attendee: Address, role: B256) -> Result<(), Vec<u8>> {
        self._remove_assignee_from_role(role, attendee)?;
        self._remove_role_from_assignee(attendee, role)?;
        Ok(())
    }

    fn _remove_role(&mut self, role: B256) -> Result<(), Vec<u8>> {
        let assignees = self._get_role_assignees(role)?;
        for assignee in assignees {
            self._remove_role_from_assignee(assignee, role)?;
        }
        Ok(())
    }
}

#[storage]
struct Delegate {
    contract_address: StorageAddress,
    required_role: StorageB256,
    access_mode: StorageU8,
}

/// Lightweight One-To-Many Proxy (aka Diamond)
///
/// This is Castle, where you are assigned roles to access functions.
///
/// Some functions are public and availble to anyone, and other functions are
/// protected and only invdividuals assigned specific role can access them.
///
/// One individual can have multiple roles assigned, and one role can be
/// assigned to multiple individuals.
///
/// Use of ACL at the Castle level partly removes the necessity to control
/// access by delegates.
/// 
///
#[entrypoint]
#[storage]
struct Castle {
    delegates: StorageMap<B32, Delegate>,
    acl: AccessControlList,
}

impl Castle {
    fn _publish_event<T>(&self, event: T)
    where
        T: SolEvent,
    {
        self.vm().emit_log(&event.encode_data(), 1);
    }

    fn _attendee(&self) -> Address {
        self.vm().msg_sender()
    }

    fn _only_admin(&self) -> Result<(), Vec<u8>> {
        self._only_role(&CASTLE_ADMIN_ROLE)?;
        Ok(())
    }

    fn _only_role(&self, role: &[u8; 32]) -> Result<(), Vec<u8>> {
        if !self.acl._has_role(role, self._attendee()) {
            Err(b"Unauthorised access")?
        }
        Ok(())
    }

    fn _prohibit_self(&self, contract_address: &Address) -> Result<(), Vec<u8>> {
        if self.vm().contract_address().eq(contract_address) {
            Err(b"Cannot reference self")?;
        }
        Ok(())
    }

    fn _is_prohibited_function(&self, fun_sel: &[u8; 4]) -> bool {
        match fun_sel {
            &ICastle::createProtectedFunctionsCall::SELECTOR
            | &ICastle::createPublicFunctionsCall::SELECTOR
            | &ICastle::removeFunctionsCall::SELECTOR
            | &ICastle::getFunctionDelegatesCall::SELECTOR
            | &ICastle::hasRoleCall::SELECTOR
            | &ICastle::grantRoleCall::SELECTOR
            | &ICastle::revokeRoleCall::SELECTOR
            | &ICastle::renounceRoleCall::SELECTOR
            | &ICastle::deleteRoleCall::SELECTOR
            | &ICastle::getAdminRoleCall::SELECTOR
            | &ICastle::getAssignedRolesCall::SELECTOR
            | &ICastle::getRoleAssigneesCall::SELECTOR => true,
            _ => false,
        }
    }

    fn _set_functions(
        &mut self,
        contract_address: Option<Address>,
        required_role: Option<B256>,
        fun_selectors: &Vec<B32>,
    ) -> Result<(), Vec<u8>> {
        contract_address.map_or(Ok(()), |x| self._prohibit_self(&x))?;
        for fun_sel in fun_selectors {
            if self._is_prohibited_function(fun_sel) {
                Err(b"Function cannot be delegated")?
            }
            let mut delegate = self.delegates.setter(*fun_sel);
            log_msg!(
                "Assigning function {} delegation to {} (previously assigned to {})",
                fun_sel,
                contract_address,
                role.contract_address.get(),
            );
            if let Some(contract_address) = contract_address {
                delegate.contract_address.set(contract_address);
            } else {
                delegate.contract_address.erase();
            }
            if let Some(required_role) = required_role {
                delegate.access_mode.set(U8::from(ACCESS_MODE_PROTECTED));
                delegate.required_role.set(required_role);
            } else {
                delegate.access_mode.set(U8::from(ACCESS_MODE_NONE));
                delegate.required_role.erase();
            }
        }
        Ok(())
    }
}

#[public]
impl Castle {
    #[constructor]
    pub fn constructor(&mut self, admin: Address) -> Result<(), Vec<u8>> {
        log_msg!("Castle administrated by {}", admin);
        self.acl._set_role(admin, CASTLE_ADMIN_ROLE.into())?;
        Ok(())
    }

    /// Associate function selectors with delegate setting **public** access.
    ///
    /// Parameters
    /// ----------
    /// - contract_address: An address of the contract implementing the functions.
    /// - fun_selectors: A list of function selectors (first 4 bytes of EVM ABI call encoding).
    ///
    /// Everyone will be able to access listed functions.
    ///
    pub fn create_public_functions(
        &mut self,
        contract_address: Address,
        function_selectors: Vec<B32>,
    ) -> Result<(), Vec<u8>> {
        self._only_admin()?;
        self._set_functions(Some(contract_address), None, &function_selectors)?;
        self._publish_event(ICastle::PublicFunctionsCreated {
            contract_address,
            function_selectors,
        });
        Ok(())
    }

    /// Associate function selectors with delegate setting **protected** access.
    ///
    /// Parameters
    /// ----------
    /// - contract_address: An address of the contract implementing the functions.
    /// - fun_selectors: A list of function selectors (first 4 bytes of EVM ABI call encoding).
    /// - required_rold: A role required to invoke any of the listed functions.
    ///
    /// Only users added to the role will be able to access listed functions.
    ///
    pub fn create_protected_functions(
        &mut self,
        contract_address: Address,
        function_selectors: Vec<B32>,
        required_role: B256,
    ) -> Result<(), Vec<u8>> {
        self._only_admin()?;
        self._set_functions(
            Some(contract_address),
            Some(required_role),
            &function_selectors,
        )?;
        self._publish_event(ICastle::ProtectedFunctionsCreated {
            contract_address,
            function_selectors,
        });
        Ok(())
    }

    /// Disassociate function selectors from delegates.
    ///
    /// Parameters
    /// ----------
    /// - fun_selectors: A list of function selectors (first 4 bytes of EVM ABI call encoding).
    ///
    pub fn remove_functions(&mut self, function_selectors: Vec<B32>) -> Result<(), Vec<u8>> {
        self._only_admin()?;
        self._set_functions(None, None, &function_selectors)?;
        self._publish_event(ICastle::FunctionsRemoved { function_selectors });
        Ok(())
    }

    /// Obtain list of implementations assigned to specific roles.
    ///
    /// Parameters
    /// ----------
    /// - fun_selectors: A list of function selectors (first 4 bytes of EVM ABI call encoding).
    ///
    pub fn get_function_delegates(&self, fun_selectors: Vec<B32>) -> Result<Vec<Address>, Vec<u8>> {
        let mut delegates = Vec::new();
        for fun_sel in &fun_selectors {
            let role = self.delegates.get(*fun_sel);
            delegates.push(role.contract_address.get());
        }
        Ok(delegates)
    }

    /// IAccessControl::hasRole()
    pub fn has_role(&mut self, role: B256, attendee: Address) -> bool {
        self.acl._has_role(&role, attendee)
    }

    // IAccessControl::grantRole()
    pub fn grant_role(&mut self, role: B256, attendee: Address) -> Result<(), Vec<u8>> {
        self._only_admin()?;
        self.acl._set_role(attendee, role)?;
        self._publish_event(ICastle::RoleGranted {
            role,
            assignee_address: attendee,
        });
        Ok(())
    }

    // IAccessControl::revokeRole()
    pub fn revoke_role(&mut self, role: B256, attendee: Address) -> Result<(), Vec<u8>> {
        self._only_admin()?;
        self.acl._unset_role(attendee, role)?;
        self._publish_event(ICastle::RoleRevoked {
            role,
            assignee_address: attendee,
        });
        Ok(())
    }

    // IAccessControl::renounceRole()
    pub fn renounce_role(&mut self, role: B256, attendee: Address) -> Result<(), Vec<u8>> {
        if self._attendee().ne(&attendee) {
            Err(b"Bad confirmation")?;
        }
        self.acl._unset_role(attendee, role)?;
        self._publish_event(ICastle::RoleRenounced {
            role,
            assignee_address: attendee,
        });
        Ok(())
    }

    /// Remove role completely with all asignees
    pub fn delete_role(&mut self, role: B256) -> Result<(), Vec<u8>> {
        self._only_admin()?;
        self.acl._remove_role(role)?;
        self._publish_event(ICastle::RoleDeleted { role });
        Ok(())
    }

    /// Return role of the Castle admin
    fn get_admin_role(&self) -> B256 {
        CASTLE_ADMIN_ROLE.into()
    }

    /// List roles assigned to attendee
    pub fn get_assigned_roles(&self, attendee: Address) -> Result<Vec<B256>, Vec<u8>> {
        self.acl._get_assigned_roles(attendee)
    }

    /// List assignees of the role
    pub fn get_role_assignees(&self, role: B256) -> Result<Vec<Address>, Vec<u8>> {
        self.acl._get_role_assignees(role)
    }

    #[payable]
    #[fallback]
    fn fallback(&mut self, calldata: &[u8]) -> ArbResult {
        let fun_sel = B32::from_slice(calldata.get(0..4).ok_or_else(|| b"Calldata invalid")?);
        let delegate = self.delegates.get(fun_sel);

        let contract_address = delegate.contract_address.get();
        if contract_address == Address::ZERO {
            log_msg!("Function {} not found", fun_sel);
            Err(b"Function not found")?;
        }

        let mode = delegate.access_mode.get();
        match mode.to::<u8>() {
            ACCESS_MODE_PROTECTED => {
                let required_role = delegate.required_role.get();
                log_msg!("Required role {} to access {}", required_role, fun_sel);
                self._only_role(&required_role)?
            }
            ACCESS_MODE_NONE => {
                // Role is public.
            }
            _ => Err(b"Invalid access mode")?,
        };

        log_msg!("Delegating function {} to {}", fun_sel, contract_address);
        unsafe { Ok(self.vm().delegate_call(&self, contract_address, calldata)?) }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_castle() {
        // TODO
    }
}