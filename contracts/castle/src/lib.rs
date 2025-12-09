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
    event CastleProtectedFunctionsCreated(address _address, bytes4[] roles);
    event CastlePublicFunctionsCreated(address _address, bytes4[] roles);
    event CastleFunctionsRemoved(bytes4[] roles);
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
struct Delegate {
    contract_address: StorageAddress,
    required_role: StorageB256,
    access_mode: StorageU8,
}

pub const ACCESS_MODE_NONE: u8 = 0;
pub const ACCESS_MODE_PROTECTED: u8 = 1;

/// One to Many proxy (aka Diamond)
///
/// All calls go through access control check.
///
/// Note: This is lightweight variant of ERC2535.
///
#[entrypoint]
#[storage]
struct Castle {
    access: AccessControl,
    access_enumerable: AccessControlEnumerable,
    delegates: StorageMap<B32, Delegate>,
}

impl Castle {
    fn _set_functions(
        &mut self,
        contract_address: Option<Address>,
        required_role: Option<B256>,
        fun_selectors: &Vec<B32>,
    ) -> Result<(), Vec<u8>> {
        self.only_role(AccessControl::DEFAULT_ADMIN_ROLE.into())?;
        if contract_address.map_or(false, |x| x == self.vm().contract_address()) {
            Err(b"Expected delegate address")?;
        }
        for fun_sel in fun_selectors {
            let mut delegate = self.delegates.setter(*fun_sel);
            log_msg!(
                "Assigning function {} delegation to {} (previously assigned to {})",
                fun_sel,
                contract_address,
                role.contract_address.get(),
            );
            if let Some(contract_address) = contract_address {
                delegate.contract_address.set(contract_address);
            }
            else {
                delegate.contract_address.erase();
            }
            if let Some(required_role) = required_role {
                delegate.access_mode.set(U8::from(ACCESS_MODE_PROTECTED));
                delegate.required_role.set(required_role);
            }
            else {
                delegate.access_mode.set(U8::from(ACCESS_MODE_NONE));
                delegate.required_role.erase();
            }
        }
        Ok(())
    }
}

#[public]
#[implements(IAccessControl<Error = control::Error>, IAccessControlEnumerable<Error = control::extensions::enumerable::Error>, IErc165)]
impl Castle {
    #[constructor]
    pub fn constructor(&mut self, admin: Address) -> Result<(), Vec<u8>> {
        log_msg!("Castle administrated by {}", admin);
        self.access_enumerable._grant_role(
            AccessControl::DEFAULT_ADMIN_ROLE.into(),
            admin,
            &mut self.access,
        );
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
    pub fn set_protected_functions(
        &mut self,
        contract_address: Address,
        fun_selectors: Vec<B32>,
        required_role: B256,
    ) -> Result<(), Vec<u8>> {
        self._set_functions(Some(contract_address), Some(required_role), &fun_selectors)?;
        self.vm().emit_log(
            &CastleProtectedFunctionsCreated {
                _address: contract_address,
                roles: fun_selectors,
            }
            .encode_data(),
            1,
        );
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
    pub fn set_public_functions(
        &mut self,
        contract_address: Address,
        fun_selectors: Vec<B32>,
    ) -> Result<(), Vec<u8>> {
        self._set_functions(Some(contract_address), None, &fun_selectors)?;
        self.vm().emit_log(
            &CastlePublicFunctionsCreated {
                _address: contract_address,
                roles: fun_selectors,
            }
            .encode_data(),
            1,
        );
        Ok(())
    }

    /// Disassociate function selectors from delegates.
    ///
    /// Parameters
    /// ----------
    /// - fun_selectors: A list of function selectors (first 4 bytes of EVM ABI call encoding).
    ///
    pub fn remove_functions(&mut self, fun_selectors: Vec<B32>) -> Result<(), Vec<u8>> {
        self._set_functions(None, None, &fun_selectors)?;
        self.vm()
            .emit_log(&CastleFunctionsRemoved { roles: fun_selectors }.encode_data(), 1);
        Ok(())
    }

    /// Obtain list of implementations assigned to specific roles.
    ///
    /// Parameters
    /// ----------
    /// - fun_selectors: A list of function selectors (first 4 bytes of EVM ABI call encoding).
    ///
    pub fn get_delegates(&self, fun_selectors: Vec<B32>) -> Result<Vec<Address>, Vec<u8>> {
        let mut delegates = Vec::new();
        for fun_sel in &fun_selectors {
            let role = self.delegates.get(*fun_sel);
            delegates.push(role.contract_address.get());
        }
        Ok(delegates)
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
                self.access.only_role(required_role)?
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
