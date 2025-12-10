// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{aliases::B32, uint, Address, B256, U256, U8};

use alloy_sol_types::{SolCall, SolEvent};
use deli::{contracts::ICastle, log_msg, storage::StorageSlot};
use stylus_sdk::{
    keccak_const,
    prelude::*,
    storage::{
        StorageAddress, StorageB256, StorageMap, StorageU256, StorageU8, StorageVec,
    },
    ArbResult,
};

pub const CASTLE_ADMIN_ROLE: [u8; 32] = keccak_const::Keccak256::new()
    .update(b"Castle.ADMIN_ROLE")
    .finalize();

pub const CASTLE_STORAGE_SLOT: U256 = {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"Castle.STORAGE_SLOT")
        .finalize();
    U256::from_be_bytes(HASH).wrapping_sub(uint!(1_U256))
};

pub const ACCESS_MODE_NONE: u8 = 0;
pub const ACCESS_MODE_PROTECTED: u8 = 1;

#[storage]
struct Role {
    assignees: StorageVec<StorageAddress>,
    positions: StorageMap<Address, StorageU256>,
}

impl Role {
    fn _assign(&mut self, address: Address) -> Result<(), Vec<u8>> {
        let mut pos_setter = self.positions.setter(address);
        if pos_setter.get().is_zero() {
            self.assignees.push(address);
            let last_pos = self.assignees.len();
            pos_setter.set(U256::from(last_pos));
        } else {
            Err(b"Role already set")?;
        }
        Ok(())
    }

    fn _unassign(&mut self, address: Address) -> Result<(), Vec<u8>> {
        let mut pos_setter = self.positions.setter(address);
        let pos = pos_setter.get();
        if pos.is_zero() {
            Err(b"Role not assigned")?;
        }
        pos_setter.erase();
        let last_index = U256::from(self.assignees.len());
        if U256::ONE < last_index && pos != last_index {
            let last = self.assignees.get(last_index - U256::ONE).unwrap();
            self.assignees.setter(pos - U256::ONE).unwrap().set(last);
            self.positions.setter(last).set(pos);
        }
        self.assignees.erase_last();
        Ok(())
    }

    fn _erase_next(&mut self, max_len: usize) -> bool {
        let assignees = self._get_assignees(0, max_len);
        for address in assignees {
            self._unassign(address).unwrap();
        }
        self.assignees.is_empty()
    }

    fn _contains(&self, address: Address) -> bool {
        !self.positions.get(address).is_zero()
    }

    fn _get_assignee_count(&self) -> usize {
        self.assignees.len()
    }

    fn _get_assignees(&self, start_from: usize, max_len: usize) -> Vec<Address> {
        let mut result = Vec::with_capacity(max_len);
        let last_index = self.assignees.len();
        if start_from < last_index {
            for index in start_from..last_index {
                let assignee = self.assignees.get(index).unwrap();
                result.push(assignee);
            }
        }
        result
    }
}

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
    roles: StorageMap<B256, Role>,
}

impl AccessControlList {
    const MAX_LEN: usize = 256;

    fn _set_role(&mut self, attendee: Address, role: B256) -> Result<(), Vec<u8>> {
        self.roles.setter(role)._assign(attendee)?;
        Ok(())
    }

    fn _has_role(&self, role: B256, attendee: Address) -> bool {
        self.roles.get(role)._contains(attendee)
    }

    fn _unset_role(&mut self, attendee: Address, role: B256) -> Result<(), Vec<u8>> {
        self.roles.setter(role)._unassign(attendee)?;
        Ok(())
    }

    fn _delete_role(&mut self, role: B256) -> bool {
        self.roles.setter(role)._erase_next(Self::MAX_LEN)
    }

    fn _get_role_assignee_count(&self, role: B256) -> usize {
        self.roles.get(role)._get_assignee_count()
    }

    fn _get_role_assignees(&self, role: B256, start_from: usize, max_len: usize) -> Vec<Address> {
        self.roles.get(role)._get_assignees(start_from, max_len.min(Self::MAX_LEN))
    }
}

#[storage]
struct Delegate {
    contract_address: StorageAddress,
    required_role: StorageB256,
    access_mode: StorageU8,
}

#[storage]
struct CastleStorage {
    delegates: StorageMap<B32, Delegate>,
    acl: AccessControlList,
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
struct Castle;

struct CastleInstanceMut<'a> {
    castle: &'a mut Castle,
    storage: CastleStorage,
}

impl Castle {
    fn _storage() -> CastleStorage {
        StorageSlot::get_slot::<CastleStorage>(CASTLE_STORAGE_SLOT)
    }

    fn _with_storage_mut<'a>(&'a mut self) -> CastleInstanceMut<'a> {
        CastleInstanceMut {
            castle: self,
            storage: Self::_storage(),
        }
    }
}

impl<'a> CastleInstanceMut<'a> {
    fn vm(&self) -> &dyn Host {
        self.castle.vm()
    }

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
        self._only_role(CASTLE_ADMIN_ROLE.into())?;
        Ok(())
    }

    fn _only_role(&self, role: B256) -> Result<(), Vec<u8>> {
        if !self.storage.acl._has_role(role, self._attendee()) {
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
            | &ICastle::getRoleAssigneeCountCall::SELECTOR
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
            let mut delegate = self.storage.delegates.setter(*fun_sel);
            if let Some(contract_address) = contract_address {
                log_msg!(
                    "Assigning function {} delegation to {} (previously assigned to {})",
                    fun_sel,
                    contract_address,
                    delegate.contract_address.get()
                );
                delegate.contract_address.set(contract_address);
            } else {
                log_msg!(
                    "Removing function {} delegation (previously assigned to {})",
                    fun_sel,
                    delegate.contract_address.get()
                );
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
        self._with_storage_mut()
            .storage
            .acl
            ._set_role(admin, CASTLE_ADMIN_ROLE.into())?;
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
        let mut with_storage = self._with_storage_mut();
        with_storage._only_admin()?;
        with_storage._set_functions(Some(contract_address), None, &function_selectors)?;
        with_storage._publish_event(ICastle::PublicFunctionsCreated {
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
    /// - required_role: A role required to invoke any of the listed functions.
    ///
    /// Only users added to the role will be able to access listed functions.
    ///
    pub fn create_protected_functions(
        &mut self,
        contract_address: Address,
        function_selectors: Vec<B32>,
        required_role: B256,
    ) -> Result<(), Vec<u8>> {
        let mut with_storage = self._with_storage_mut();
        with_storage._only_admin()?;
        with_storage._set_functions(
            Some(contract_address),
            Some(required_role),
            &function_selectors,
        )?;
        with_storage._publish_event(ICastle::ProtectedFunctionsCreated {
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
        let mut with_storage = self._with_storage_mut();
        with_storage._only_admin()?;
        with_storage._set_functions(None, None, &function_selectors)?;
        with_storage._publish_event(ICastle::FunctionsRemoved { function_selectors });
        Ok(())
    }

    /// Obtain list of implementations assigned to specific roles.
    ///
    /// Parameters
    /// ----------
    /// - fun_selectors: A list of function selectors (first 4 bytes of EVM ABI call encoding).
    ///
    pub fn get_function_delegates(&self, fun_selectors: Vec<B32>) -> Result<Vec<Address>, Vec<u8>> {
        let storage = Self::_storage();
        let mut delegates = Vec::new();
        for fun_sel in &fun_selectors {
            let role = storage.delegates.get(*fun_sel);
            delegates.push(role.contract_address.get());
        }
        Ok(delegates)
    }

    /// IAccessControl::hasRole()
    pub fn has_role(&mut self, role: B256, attendee: Address) -> bool {
        Self::_storage().acl._has_role(role, attendee)
    }

    // IAccessControl::grantRole()
    pub fn grant_role(&mut self, role: B256, attendee: Address) -> Result<(), Vec<u8>> {
        let mut with_storage = self._with_storage_mut();
        with_storage._only_admin()?;
        with_storage.storage.acl._set_role(attendee, role)?;
        with_storage._publish_event(ICastle::RoleGranted {
            role,
            assignee_address: attendee,
        });
        Ok(())
    }

    // IAccessControl::revokeRole()
    pub fn revoke_role(&mut self, role: B256, attendee: Address) -> Result<(), Vec<u8>> {
        let mut with_storage = self._with_storage_mut();
        with_storage._only_admin()?;
        with_storage.storage.acl._unset_role(attendee, role)?;
        with_storage._publish_event(ICastle::RoleRevoked {
            role,
            assignee_address: attendee,
        });
        Ok(())
    }

    // IAccessControl::renounceRole()
    pub fn renounce_role(&mut self, role: B256, attendee: Address) -> Result<(), Vec<u8>> {
        let mut with_storage = self._with_storage_mut();
        if with_storage._attendee().ne(&attendee) {
            Err(b"Bad confirmation")?;
        }
        with_storage.storage.acl._unset_role(attendee, role)?;
        with_storage._publish_event(ICastle::RoleRenounced {
            role,
            assignee_address: attendee,
        });
        Ok(())
    }

    /// Remove role completely with all asignees
    pub fn delete_role(&mut self, role: B256) -> Result<bool, Vec<u8>> {
        let mut with_storage = self._with_storage_mut();
        with_storage._only_admin()?;
        let deleted = with_storage.storage.acl._delete_role(role);
        if deleted {
            with_storage._publish_event(ICastle::RoleDeleted { role });
        }
        Ok(deleted)
    }

    /// Return role of the Castle admin
    fn get_admin_role(&self) -> B256 {
        CASTLE_ADMIN_ROLE.into()
    }

    pub fn get_role_assignee_count(&self, role: B256) -> Result<U256, Vec<u8>> {
        let assignee_count = Self::_storage().acl._get_role_assignee_count(role);
        Ok(U256::from(assignee_count))
    }

    /// List assignees of the role
    pub fn get_role_assignees(
        &self,
        role: B256,
        start_from: U256,
        max_len: U256,
    ) -> Result<Vec<Address>, Vec<u8>> {
        let assignees =
            Self::_storage()
                .acl
                ._get_role_assignees(role, start_from.to(), max_len.to());
        Ok(assignees)
    }

    #[payable]
    #[fallback]
    fn fallback(&mut self, calldata: &[u8]) -> ArbResult {
        let with_storage = self._with_storage_mut();
        let fun_sel = B32::from_slice(calldata.get(0..4).ok_or_else(|| b"Calldata invalid")?);
        let delegate = with_storage.storage.delegates.get(fun_sel);

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
                with_storage._only_role(required_role)?
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
