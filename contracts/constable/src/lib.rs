// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::Address;
use alloy_sol_types::SolCall;
use deli::contracts::{
    castle::CASTLE_ADMIN_ROLE,
    interfaces::{
        banker::IBanker,
        castle::ICastle::{self},
        constable::IConstable,
        factor::IFactor,
        guildmaster::IGuildmaster,
        scribe::IScribe::{self, IScribeCalls},
        worksman::IWorksman,
    },
    keep::Keep,
};
use stylus_sdk::{keccak_const, prelude::*};

pub const CASTLE_ISSUER_ROLE: [u8; 32] = keccak_const::Keccak256::new()
    .update(b"Castle.ISSUER_ROLE")
    .finalize();

pub const CASTLE_VENDOR_ROLE: [u8; 32] = keccak_const::Keccak256::new()
    .update(b"Castle.VENDOR_ROLE")
    .finalize();

pub const CASTLE_KEEPER_ROLE: [u8; 32] = keccak_const::Keccak256::new()
    .update(b"Castle.KEEPER_ROLE")
    .finalize();

#[storage]
#[entrypoint]
pub struct Constable;

impl Constable {
    fn _dispatch(&mut self, castle: Address, call: impl SolCall) -> Result<Vec<u8>, Vec<u8>> {
        let calldata = call.abi_encode();
        let result = unsafe { self.vm().delegate_call(&self, castle, &calldata) }?;
        Ok(result)
    }
}

#[public]
impl Constable {
    pub fn accept_appointment(&mut self, castle: Address) -> Result<(), Vec<u8>> {
        let mut storage = Keep::storage();
        if !storage.constable.get().is_zero() {
            Err(b"Constable already appointed")?;
        }
        storage.initialize(castle, self.vm().contract_address());
        let constable_role = ICastle::createProtectedFunctionsCall {
            contract_address: castle,
            function_selectors: vec![
                IConstable::appointWorksmanCall::SELECTOR.into(),
                IConstable::castRolesCall::SELECTOR.into(),
            ],
            required_role: CASTLE_ADMIN_ROLE.into(),
        };
        self._dispatch(castle, constable_role)?;
        Ok(())
    }

    pub fn appoint_worksman(&mut self, worksman: Address) -> Result<(), Vec<u8>> {
        let storage = Keep::storage();
        let castle = storage.castle.get();
        if castle.is_zero() {
            Err(b"Constable was not appointed")?;
        }
        let calldata = IWorksman::acceptAppointmentCall { castle }.abi_encode();
        unsafe { self.vm().delegate_call(&self, worksman, &calldata) }?;
        let build_vault_role = ICastle::createProtectedFunctionsCall {
            contract_address: castle,
            function_selectors: vec![IWorksman::buildVaultCall::SELECTOR.into()],
            required_role: CASTLE_ISSUER_ROLE.into(),
        };
        self._dispatch(castle, build_vault_role)?;
        Ok(())
    }

    pub fn appoint_scribe(&mut self, scribe: Address) -> Result<(), Vec<u8>> {
        let storage = Keep::storage();
        let castle = storage.castle.get();
        if castle.is_zero() {
            Err(b"Constable was not appointed")?;
        }
        let calldata = IScribe::acceptAppointmentCall { castle }.abi_encode();
        unsafe { self.vm().delegate_call(&self, scribe, &calldata) }?;
        let scribe_role = ICastle::createProtectedFunctionsCall {
            contract_address: castle,
            function_selectors: vec![IScribe::verifySignatureCall::SELECTOR.into()],
            required_role: CASTLE_ISSUER_ROLE.into(),
        };
        self._dispatch(castle, scribe_role)?;
        Ok(())
    }

    pub fn cast_roles(
        &mut self,
        guildmaster: Address,
        banker: Address,
        factor: Address,
        gate_to_granary: Address,
    ) -> Result<(), Vec<u8>> {
        let mut storage = Keep::storage();
        let castle = storage.castle.get();
        if castle.is_zero() {
            Err(b"Constable was not appointed")?;
        }
        if storage.constable.get() != self.vm().contract_address() {
            Err(b"Wrong Castle")?;
        }
        if !storage.granary.get_granary_address().is_zero() {
            Err(b"Granary already cast")?;
        }
        storage.granary.initialize(gate_to_granary);

        let issuer_role = ICastle::createProtectedFunctionsCall {
            contract_address: guildmaster,
            function_selectors: vec![
                IGuildmaster::submitIndexCall::SELECTOR.into(),
                IGuildmaster::submitVoteCall::SELECTOR.into(),
            ],
            required_role: CASTLE_ISSUER_ROLE.into(),
        };
        let vendor_role_1 = ICastle::createProtectedFunctionsCall {
            contract_address: banker,
            function_selectors: vec![
                IBanker::submitAssetsCall::SELECTOR.into(),
                IBanker::submitMarginCall::SELECTOR.into(),
                IBanker::submitSupplyCall::SELECTOR.into(),
            ],
            required_role: CASTLE_VENDOR_ROLE.into(),
        };
        let vendor_role_2 = ICastle::createProtectedFunctionsCall {
            contract_address: factor,
            function_selectors: vec![IFactor::submitMarketDataCall::SELECTOR.into()],
            required_role: CASTLE_VENDOR_ROLE.into(),
        };
        let keeper_role = ICastle::createProtectedFunctionsCall {
            contract_address: factor,
            function_selectors: vec![
                IFactor::updateIndexQuoteCall::SELECTOR.into(),
                IFactor::updateMultipleIndexQuotesCall::SELECTOR.into(),
            ],
            required_role: CASTLE_KEEPER_ROLE.into(),
        };
        let trader_role = ICastle::createPublicFunctionsCall {
            contract_address: factor,
            function_selectors: vec![IFactor::submitBuyOrderCall::SELECTOR.into()],
        };
        self._dispatch(castle, issuer_role)?;
        self._dispatch(castle, vendor_role_1)?;
        self._dispatch(castle, vendor_role_2)?;
        self._dispatch(castle, keeper_role)?;
        self._dispatch(castle, trader_role)?;
        Ok(())
    }
}
