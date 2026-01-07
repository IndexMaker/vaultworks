// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{aliases::B32, Address, B256, U256};

use alloy_sol_types::SolCall;
use alloy_sol_types::SolEvent;
use common::log_msg;
use common_contracts::{
    contracts::{
        acl::AccessControlList,
        calls::InnerCall,
        castle::{CastleStorage, CASTLE_ADMIN_ROLE},
    },
    interfaces::{castle::ICastle, constable::IConstable},
};
use stylus_sdk::{prelude::*, ArbResult};

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

    fn _only_admin(&self, acl: &AccessControlList) -> Result<(), Vec<u8>> {
        acl.only_role(self.get_admin_role(), self._attendee())?;
        Ok(())
    }

    fn _prohibit_self(&self, contract_address: &Address) -> Result<(), Vec<u8>> {
        if contract_address.is_zero() {
            Err(b"Cannot reference null contract")?;
        }
        if self.vm().contract_address().eq(contract_address) {
            Err(b"Cannot reference self")?;
        }
        Ok(())
    }

    fn _is_prohibited_function(&self, fun_sel: &[u8; 4]) -> bool {
        match fun_sel {
            &ICastle::appointConstableCall::SELECTOR
            | &ICastle::createProtectedFunctionsCall::SELECTOR
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

    fn _check_functions(&self, fun_selectors: &Vec<B32>) -> Result<(), Vec<u8>> {
        for fun_sel in fun_selectors {
            if self._is_prohibited_function(fun_sel) {
                Err(b"Function cannot be delegated")?
            }
        }
        Ok(())
    }
}

#[public]
impl Castle {
    // TODO: Consider whether to add UUPS (ERC-1967) if Castle itself needs to be upgradeable.
    // Probably yes, because there is a lot of logic here, so would be good to
    // be able to patch when needed. We'd essentially put Castle behind the Gate.

    #[constructor]
    pub fn constructor(&mut self) -> Result<(), Vec<u8>> {
        let mut storage = CastleStorage::storage();
        storage.construct(self.vm().contract_address())?;
        Ok(())
    }

    pub fn initialize(&mut self, castle: Address, admin: Address) -> Result<(), Vec<u8>> {
        let mut storage = CastleStorage::storage();
        storage.construct(castle)?;

        log_msg!("Castle administrated by {}", admin);
        storage
            .get_acl_mut()
            .set_role(admin, CASTLE_ADMIN_ROLE.into())?;
        Ok(())
    }

    /// Appoint a Constable to cast roles in this Castle
    ///
    /// Constable logic is injected into Castle context, and after that user
    /// with appropriate privileges (ACL) will be able to call Constable method
    /// on Castle, this way that method will be able to modify storage slots associated
    /// with this Castle.
    ///
    /// Note: Constable cannot cast roles directly within acceptAppointment(), because
    /// user needs to pass additional data, and because of that it happens in separate
    /// call.
    ///
    /// Note: Appointed delegates of the Castle will be able to modify storage associated
    /// with the Castle, if and only if their methods are called on Castle context and not
    /// their own storage. It is safe undefined behaviour in case someone called method of
    /// the delegate on that delegate's context. It's safe as the storage of the Castle
    /// cannot be modified in such case, and what matters is only what is in that storage.
    ///
    pub fn appoint_constable(&mut self, constable: Address) -> Result<(), Vec<u8>> {
        let mut storage = CastleStorage::storage();
        let acl = storage.get_acl_mut();

        self._only_admin(acl)?;
        self.inner_call(constable, IConstable::acceptAppointmentCall { constable })?;
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
        self._prohibit_self(&contract_address)?;
        self._check_functions(&function_selectors)?;

        let mut storage = CastleStorage::storage();
        let acl = storage.get_acl_mut();

        self._only_admin(acl)?;
        storage.set_functions(Some(contract_address), None, &function_selectors);

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
        self._prohibit_self(&contract_address)?;
        self._check_functions(&function_selectors)?;

        let mut storage = CastleStorage::storage();
        let acl = storage.get_acl_mut();

        self._only_admin(acl)?;
        storage.set_functions(
            Some(contract_address),
            Some(required_role),
            &function_selectors,
        );

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
        self._check_functions(&function_selectors)?;

        let mut storage = CastleStorage::storage();
        let acl = storage.get_acl_mut();

        self._only_admin(acl)?;
        storage.set_functions(None, None, &function_selectors);

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
        let storage = CastleStorage::storage();
        let mut delegates = Vec::new();

        for fun_sel in fun_selectors {
            if let Some(contract_address) = storage.get_function_delegate_address(fun_sel) {
                delegates.push(contract_address);
            }
        }
        Ok(delegates)
    }

    /// IAccessControl::hasRole()
    pub fn has_role(&mut self, role: B256, attendee: Address) -> bool {
        let storage = CastleStorage::storage();
        storage.get_acl().has_role(role, attendee)
    }

    // IAccessControl::grantRole()
    pub fn grant_role(&mut self, role: B256, attendee: Address) -> Result<(), Vec<u8>> {
        let mut storage = CastleStorage::storage();
        let acl = storage.get_acl_mut();

        self._only_admin(acl)?;
        acl.set_role(attendee, role)?;

        self._publish_event(ICastle::RoleGranted {
            role,
            assignee_address: attendee,
        });

        Ok(())
    }

    // IAccessControl::revokeRole()
    pub fn revoke_role(&mut self, role: B256, attendee: Address) -> Result<(), Vec<u8>> {
        let mut storage = CastleStorage::storage();
        let acl = storage.get_acl_mut();

        self._only_admin(acl)?;
        acl.unset_role(attendee, role)?;

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

        let mut storage = CastleStorage::storage();
        storage.get_acl_mut().unset_role(attendee, role)?;

        self._publish_event(ICastle::RoleRenounced {
            role,
            assignee_address: attendee,
        });
        Ok(())
    }

    /// Remove role completely with all asignees
    pub fn delete_role(&mut self, role: B256) -> Result<bool, Vec<u8>> {
        let mut storage = CastleStorage::storage();
        let acl = storage.get_acl_mut();

        self._only_admin(acl)?;
        let deleted = acl.delete_role(role);

        if deleted {
            self._publish_event(ICastle::RoleDeleted { role });
        }

        Ok(deleted)
    }

    /// Return role of the Castle admin
    pub fn get_admin_role(&self) -> B256 {
        CASTLE_ADMIN_ROLE.into()
    }

    pub fn get_role_assignee_count(&self, role: B256) -> Result<U256, Vec<u8>> {
        let storage = CastleStorage::storage();

        let assignee_count = storage.get_acl().get_role_assignee_count(role);
        Ok(U256::from(assignee_count))
    }

    /// List assignees of the role
    pub fn get_role_assignees(
        &self,
        role: B256,
        start_from: U256,
        max_len: U256,
    ) -> Result<Vec<Address>, Vec<u8>> {
        let storage = CastleStorage::storage();

        let assignees = storage
            .get_acl()
            .get_role_assignees(role, start_from.to(), max_len.to());

        Ok(assignees)
    }

    #[payable]
    #[fallback]
    fn fallback(&mut self, calldata: &[u8]) -> ArbResult {
        let storage = CastleStorage::storage();

        let Some((contract_address, required_role)) =
            storage.get_function_delegate_from_calldata(calldata)?
        else {
            return Err(format!(
                "Function not found: 0x{}",
                hex::encode(&calldata[0..4])
            ))?;
        };

        if let Some(required_role) = required_role {
            if !required_role.contains(self._attendee()) {
                log_msg!("Unauthorised access");
                Err(b"Unauthorised access")?;
            }
        }

        log_msg!("Delegating function to {}", contract_address);
        unsafe { Ok(self.vm().delegate_call(&self, contract_address, calldata)?) }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_castle() {}
}
