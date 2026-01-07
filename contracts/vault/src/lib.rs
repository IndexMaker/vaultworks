// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::{string::String, vec::Vec};

use alloy_primitives::{uint, Address, B256, U128, U256, U32, U8};
use alloy_sol_types::{sol, SolEvent};
use common::amount::Amount;
use common_contracts::{
    contracts::{
        calls::InnerCall,
        gate::{Gate, IMPLEMENTATION_SLOT},
        keep_calls::KeepCalls,
        vault::VaultStorage,
    },
    interfaces::factor::IFactor,
};
use stylus_sdk::{abi::Bytes, prelude::*, ArbResult};

pub const VERSION_NUMBER: U32 = uint!(1_U32);

sol! {
    interface IERC20 {
        event Transfer(address indexed from, address indexed to, uint256 value);

        event Approval(address indexed owner, address indexed spender, uint256 value);

        function transferFrom(address from, address to, uint256 value) external returns (bool);
    }
}

#[storage]
#[entrypoint]
pub struct Vault;

#[public]
impl Vault {
    #[constructor]
    fn constructor(&mut self, initial_owner: Address) -> Result<(), Vec<u8>> {
        Gate::construct_logic();
        let mut vault = VaultStorage::storage();
        vault.set_owner(initial_owner)
    }

    fn initialize(
        &mut self,
        owner: Address,
        requests: Address,
        gate_to_castle: Address,
    ) -> Result<(), Vec<u8>> {
        Gate::only_delegated()?;
        let mut vault = VaultStorage::storage();
        vault.only_owner(self.attendee())?;
        vault.set_version(VERSION_NUMBER)?;
        vault.set_owner(owner)?;
        vault.set_requests(requests);
        vault.set_castle(gate_to_castle);
        Ok(())
    }

    pub fn set_version(&mut self) -> Result<(), Vec<u8>> {
        Gate::only_delegated()?;
        let mut vault = VaultStorage::storage();
        vault.only_owner(self.attendee())?;
        vault.set_version(VERSION_NUMBER)
    }

    pub fn get_version(&self) -> U32 {
        VERSION_NUMBER
    }

    // UUPS

    #[selector(name = "UPGRADE_INTERFACE_VERSION")]
    fn upgrade_interface_version(&self) -> String {
        Gate::upgrade_interface_version()
    }

    #[payable]
    pub fn upgrade_to_and_call(
        &mut self,
        new_implementation: Address,
        data: Bytes,
    ) -> Result<(), Vec<u8>> {
        let vault = VaultStorage::storage();
        vault.only_owner(self.attendee())?;
        if vault.version.get() != VERSION_NUMBER {
            Err(b"UUPSUnauthorizedCallContext")?;
        }
        Gate::upgrade_to_and_call(self, new_implementation, data)
    }

    fn proxiable_uuid(&self) -> Result<B256, Vec<u8>> {
        Ok(IMPLEMENTATION_SLOT)
    }

    // Ownable

    fn owner(&self) -> Address {
        let vault = VaultStorage::storage();
        vault.owner.get()
    }

    fn transfer_ownership(&mut self, new_owner: Address) -> Result<(), Vec<u8>> {
        let mut vault = VaultStorage::storage();
        vault.only_owner(self.attendee())?;
        vault.set_owner(new_owner)
    }

    fn renounce_ownership(&mut self) -> Result<(), Vec<u8>> {
        let mut vault = VaultStorage::storage();
        vault.only_owner(self.attendee())?;
        vault.set_owner(Address::ZERO)
    }

    pub fn configure_vault(
        &mut self,
        index_id: U128,
        name: String,
        symbol: String,
    ) -> Result<(), Vec<u8>> {
        let mut vault = VaultStorage::storage();
        vault.only_owner(self.attendee())?;
        vault.index_id.set(index_id);
        vault.name.set_str(name);
        vault.symbol.set_str(symbol);
        Ok(())
    }

    pub fn castle(&self) -> Address {
        let vault = VaultStorage::storage();
        vault.gate_to_castle.get()
    }

    pub fn index_id(&self) -> U128 {
        let vault = VaultStorage::storage();
        vault.index_id.get()
    }

    // ERC20

    pub fn name(&self) -> alloc::string::String {
        let vault = VaultStorage::storage();
        vault.name.get_string()
    }

    pub fn symbol(&self) -> alloc::string::String {
        let vault = VaultStorage::storage();
        vault.symbol.get_string()
    }

    pub fn decimals(&self) -> U8 {
        U8::from(18)
    }

    pub fn total_supply(&self) -> Result<U256, Vec<u8>> {
        let vault = VaultStorage::storage();

        let order = vault.get_total_order(self)?;
        let itp_amount = order.tell_total()?;

        Ok(itp_amount.to_u256())
    }

    pub fn balance_of(&self, account: Address) -> Result<U256, Vec<u8>> {
        let vault = VaultStorage::storage();

        let order = vault.get_order(self, account)?;
        let itp_amount = order.tell_available()?;

        Ok(itp_amount.to_u256())
    }

    pub fn transfer(&mut self, to: Address, value: U256) -> Result<(), Vec<u8>> {
        let vault = VaultStorage::storage();
        let sender = self.attendee();

        // Vault is submitting transfer on behalf of msg.sender (attendee)
        self.external_call(
            vault.gate_to_castle.get(),
            IFactor::submitTransferCall {
                index_id: vault.index_id.get().to(),
                sender,
                receiver: to,
                amount: Amount::try_from_u256(value)
                    .ok_or_else(|| b"MathOverflow")?
                    .to_u128_raw(),
            },
        )?;

        let event = IERC20::Transfer {
            from: sender,
            to,
            value,
        };

        self.vm().emit_log(&event.encode_data(), 1);

        Ok(())
    }

    pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
        let vault = VaultStorage::storage();
        let allowances = vault.allowances.get(owner);
        allowances.allowance(spender)
    }

    pub fn approve(&mut self, spender: Address, value: U256) -> Result<bool, Vec<u8>> {
        let sender = self.attendee();
        if spender.is_zero() {
            Err(b"Invalid Spender")?;
        }
        let mut vault = VaultStorage::storage();
        let mut allowance = vault.allowances.setter(sender);
        let result = allowance.approve(spender, value)?;

        let event = IERC20::Approval {
            owner: sender,
            spender,
            value,
        };

        self.vm().emit_log(&event.encode_data(), 1);

        Ok(result)
    }

    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Vec<u8>> {
        if from.is_zero() {
            Err(b"Invalid Spender")?;
        }
        if to.is_zero() {
            Err(b"Invalid Receiver")?;
        }
        let mut vault = VaultStorage::storage();
        let mut allowance = vault.allowances.setter(self.attendee());
        allowance.spend_allowance(from, value)?;

        self.external_call(
            vault.gate_to_castle.get(),
            IFactor::submitTransferCall {
                index_id: vault.index_id.get().to(),
                sender: from,
                receiver: to,
                amount: Amount::try_from_u256(value)
                    .ok_or_else(|| b"MathOverflow")?
                    .to_u128_raw(),
            },
        )?;

        let event = IERC20::Transfer { from, to, value };

        self.vm().emit_log(&event.encode_data(), 1);
        Ok(true)
    }

    #[payable]
    #[fallback]
    fn fallback(&mut self, calldata: &[u8]) -> ArbResult {
        let requests = {
            let vault = VaultStorage::storage();
            let requests = vault.requests_implementation.get();
            if requests.is_zero() {
                Err(b"No requests implementation")?;
            }
            requests
        };

        unsafe {
            let result = self.vm().delegate_call(&self, requests, calldata)?;
            Ok(result)
        }
    }
}
