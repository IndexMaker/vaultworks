// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

const TOKEN_NAME: &str = "IndexMaker Token";
const TOKEN_SYMBOL: &str = "IM";

use alloc::{string::String, vec::Vec};

use alloy_primitives::{uint, U32, U8};
use alloy_sol_types::SolCall;
use openzeppelin_stylus::{
    access::ownable::{self, IOwnable, Ownable},
    proxy::{
        erc1967::{
            self,
            utils::{ERC1967InvalidImplementation, IMPLEMENTATION_SLOT},
            Erc1967Utils,
        },
        utils::{
            erc1822::{Erc1822ProxiableInterface, IErc1822Proxiable},
            uups_upgradeable::{
                self, IUUPSUpgradeable, InvalidVersion, UUPSUnauthorizedCallContext,
                UUPSUnsupportedProxiableUUID, UUPSUpgradeableInterface, LOGIC_FLAG_SLOT,
                UPGRADE_INTERFACE_VERSION,
            },
        },
    },
    token::erc20::{
        self,
        extensions::{Erc20Metadata, IErc20Metadata},
        Erc20, IErc20,
    },
    utils::{
        address::{self, AddressUtils},
        storage_slot::StorageSlot,
    },
};
#[allow(deprecated)]
use stylus_sdk::{
    abi::Bytes,
    alloy_primitives::{Address, B256, U256},
    call::Call,
    prelude::*,
    storage::{StorageBool, StorageU32},
};

#[derive(SolidityError, Debug)]
enum Error {
    UnauthorizedAccount(ownable::OwnableUnauthorizedAccount),
    InvalidOwner(ownable::OwnableInvalidOwner),
    UnauthorizedCallContext(uups_upgradeable::UUPSUnauthorizedCallContext),
    UnsupportedProxiableUUID(uups_upgradeable::UUPSUnsupportedProxiableUUID),
    InvalidImplementation(erc1967::utils::ERC1967InvalidImplementation),
    InvalidAdmin(erc1967::utils::ERC1967InvalidAdmin),
    InvalidBeacon(erc1967::utils::ERC1967InvalidBeacon),
    NonPayable(erc1967::utils::ERC1967NonPayable),
    EmptyCode(address::AddressEmptyCode),
    FailedCall(address::FailedCall),
    FailedCallWithReason(address::FailedCallWithReason),
    InvalidInitialization(uups_upgradeable::InvalidInitialization),
    InvalidVersion(uups_upgradeable::InvalidVersion),
}

impl From<uups_upgradeable::Error> for Error {
    fn from(e: uups_upgradeable::Error) -> Self {
        match e {
            uups_upgradeable::Error::InvalidImplementation(e) => Error::InvalidImplementation(e),
            uups_upgradeable::Error::InvalidAdmin(e) => Error::InvalidAdmin(e),
            uups_upgradeable::Error::InvalidBeacon(e) => Error::InvalidBeacon(e),
            uups_upgradeable::Error::NonPayable(e) => Error::NonPayable(e),
            uups_upgradeable::Error::EmptyCode(e) => Error::EmptyCode(e),
            uups_upgradeable::Error::FailedCall(e) => Error::FailedCall(e),
            uups_upgradeable::Error::FailedCallWithReason(e) => Error::FailedCallWithReason(e),
            uups_upgradeable::Error::InvalidInitialization(e) => Error::InvalidInitialization(e),
            uups_upgradeable::Error::UnauthorizedCallContext(e) => {
                Error::UnauthorizedCallContext(e)
            }
            uups_upgradeable::Error::UnsupportedProxiableUUID(e) => {
                Error::UnsupportedProxiableUUID(e)
            }
            uups_upgradeable::Error::InvalidVersion(e) => Error::InvalidVersion(e),
        }
    }
}

impl From<ownable::Error> for Error {
    fn from(e: ownable::Error) -> Self {
        match e {
            ownable::Error::UnauthorizedAccount(e) => Error::UnauthorizedAccount(e),
            ownable::Error::InvalidOwner(e) => Error::InvalidOwner(e),
        }
    }
}

pub const VERSION_NUMBER: U32 = uups_upgradeable::VERSION_NUMBER.wrapping_add(uint!(1_U32));

#[entrypoint]
#[storage]
struct Treasury {
    erc20: Erc20,
    erc20_medatada: Erc20Metadata,
    ownable: Ownable,
    version: StorageU32,
}

#[public]
#[implements(IErc20<Error = erc20::Error>, IUUPSUpgradeable, IErc1822Proxiable, IOwnable)]
impl Treasury {
    // Accepting owner here only to enable invoking functions directly on the
    // UUPS
    #[constructor]
    fn constructor(&mut self, owner: Address) -> Result<(), Error> {
        self.logic_flag().set(true);
        self.ownable.constructor(owner)?;
        Ok(())
    }

    fn mint(&mut self, to: Address, value: U256) -> Result<(), erc20::Error> {
        self.erc20._mint(to, value)
    }

    fn name(&self) -> String {
        self.erc20_medatada.name()
    }

    fn symbol(&self) -> String {
        self.erc20_medatada.symbol()
    }

    fn decimals(&self) -> U8 {
        self.erc20_medatada.decimals()
    }

    // --- without these the methods don't show up in exported ABI

    fn balance_of(&self, account: Address) -> U256 {
        <Self as IErc20>::balance_of(self, account)
    }

    fn total_supply(&self) -> U256 {
        <Self as IErc20>::total_supply(self)
    }

    fn transfer(&mut self, to: Address, value: U256) -> Result<bool, erc20::Error> {
        <Self as IErc20>::transfer(self, to, value)
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, erc20::Error> {
        <Self as IErc20>::transfer_from(self, from, to, value)
    }

    fn allowance(&self, owner: Address, spender: Address) -> U256 {
        <Self as IErc20>::allowance(self, owner, spender)
    }

    fn approve(&mut self, spender: Address, value: U256) -> Result<bool, erc20::Error> {
        <Self as IErc20>::approve(self, spender, value)
    }

    fn upgrade_to_and_call(
        &mut self,
        new_implementation: Address,
        data: Bytes,
    ) -> Result<(), Vec<u8>> {
        <Self as IUUPSUpgradeable>::upgrade_to_and_call(self, new_implementation, data)
    }

    fn owner(&self) -> Address {
        <Self as IOwnable>::owner(self)
    }

    fn transfer_ownership(&mut self, new_owner: Address) -> Result<(), Vec<u8>> {
        <Self as IOwnable>::transfer_ownership(self, new_owner)
    }

    fn renounce_ownership(&mut self) -> Result<(), Vec<u8>> {
        <Self as IOwnable>::renounce_ownership(self)
    }

    // ---

    fn initialize(&mut self, owner: Address) -> Result<(), Error> {
        self.set_version()?;
        self.ownable.constructor(owner)?;
        self.erc20_medatada
            .constructor(String::from(TOKEN_NAME), String::from(TOKEN_SYMBOL));
        Ok(())
    }

    fn set_version(&mut self) -> Result<(), Error> {
        if self.not_delegated().is_ok() {
            return Err(Error::UnauthorizedCallContext(
                UUPSUnauthorizedCallContext {},
            ));
        }
        if self.version.get() > VERSION_NUMBER {
            return Err(Error::InvalidVersion(InvalidVersion {
                current_version: self.version.get().to(),
            }));
        }

        self.version.set(VERSION_NUMBER);
        Ok(())
    }

    pub fn get_version(&self) -> U32 {
        VERSION_NUMBER
    }
}

#[public]
impl IUUPSUpgradeable for Treasury {
    #[selector(name = "UPGRADE_INTERFACE_VERSION")]
    fn upgrade_interface_version(&self) -> String {
        UPGRADE_INTERFACE_VERSION.into()
    }

    #[payable]
    fn upgrade_to_and_call(
        &mut self,
        new_implementation: Address,
        data: Bytes,
    ) -> Result<(), Vec<u8>> {
        self.ownable.only_owner()?;
        self.only_proxy()?;
        #[allow(clippy::used_underscore_items)]
        self._upgrade_to_and_call_uups(new_implementation, &data)?;

        let data_set_version = UUPSUpgradeableInterface::setVersionCall {}.abi_encode();
        AddressUtils::function_delegate_call(self, new_implementation, &data_set_version)?;

        Ok(())
    }
}

impl Treasury {
    pub fn logic_flag(&self) -> StorageBool {
        StorageSlot::get_slot::<StorageBool>(LOGIC_FLAG_SLOT)
    }

    pub fn is_logic(&self) -> bool {
        self.logic_flag().get()
    }

    pub fn only_proxy(&self) -> Result<(), Error> {
        if self.is_logic()
            || Erc1967Utils::get_implementation() == Address::ZERO
            || U32::from(self.get_version()) != self.version.get()
        {
            Err(Error::UnauthorizedCallContext(
                UUPSUnauthorizedCallContext {},
            ))
        } else {
            Ok(())
        }
    }

    pub fn not_delegated(&self) -> Result<(), Error> {
        if self.is_logic() {
            Ok(())
        } else {
            Err(Error::UnauthorizedCallContext(
                UUPSUnauthorizedCallContext {},
            ))
        }
    }
}

#[public]
impl IErc1822Proxiable for Treasury {
    #[selector(name = "proxiableUUID")]
    fn proxiable_uuid(&self) -> Result<B256, Vec<u8>> {
        self.not_delegated()?;
        Ok(IMPLEMENTATION_SLOT)
    }
}

impl Treasury {
    fn _upgrade_to_and_call_uups(
        &mut self,
        new_implementation: Address,
        data: &Bytes,
    ) -> Result<(), Error> {
        #[allow(deprecated)]
        let slot = Erc1822ProxiableInterface::new(new_implementation)
            .proxiable_uuid(Call::new_in(self))
            .map_err(|_e| {
                Error::InvalidImplementation(ERC1967InvalidImplementation {
                    implementation: new_implementation,
                })
            })?;

        if slot == IMPLEMENTATION_SLOT {
            Erc1967Utils::upgrade_to_and_call(self, new_implementation, data)
                .map_err(uups_upgradeable::Error::from)
                .map_err(Error::from)
        } else {
            Err(Error::UnsupportedProxiableUUID(
                UUPSUnsupportedProxiableUUID { slot },
            ))
        }
    }
}

#[public]
impl IErc20 for Treasury {
    type Error = erc20::Error;

    fn balance_of(&self, account: Address) -> U256 {
        self.erc20.balance_of(account)
    }

    fn total_supply(&self) -> U256 {
        self.erc20.total_supply()
    }

    fn transfer(&mut self, to: Address, value: U256) -> Result<bool, erc20::Error> {
        self.erc20.transfer(to, value)
    }

    fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, erc20::Error> {
        self.erc20.transfer_from(from, to, value)
    }

    fn allowance(&self, owner: Address, spender: Address) -> U256 {
        self.erc20.allowance(owner, spender)
    }

    fn approve(&mut self, spender: Address, value: U256) -> Result<bool, erc20::Error> {
        self.erc20.approve(spender, value)
    }
}

#[public]
impl IOwnable for Treasury {
    fn owner(&self) -> Address {
        self.ownable.owner()
    }

    fn transfer_ownership(&mut self, new_owner: Address) -> Result<(), Vec<u8>> {
        Ok(self.ownable.transfer_ownership(new_owner)?)
    }

    fn renounce_ownership(&mut self) -> Result<(), Vec<u8>> {
        Ok(self.ownable.renounce_ownership()?)
    }
}
