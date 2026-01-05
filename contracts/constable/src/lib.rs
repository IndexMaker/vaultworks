// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, B256};
use alloy_sol_types::SolCall;
use common::log_msg;
use common_contracts::{
    contracts::{calls::InnerCall, castle::CASTLE_ADMIN_ROLE, keep::Keep},
    interfaces::{
        banker::IBanker, castle::ICastle, constable::IConstable, factor::IFactor,
        guildmaster::IGuildmaster, scribe::IScribe, worksman::IWorksman,
    },
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

#[public]
impl Constable {
    pub fn accept_appointment(&mut self, constable: Address) -> Result<(), Vec<u8>> {
        log_msg!("Accepting appointment {}", constable);
        let mut storage = Keep::storage();
        if !storage.constable.get().is_zero() {
            Err(b"Constable already appointed")?;
        }
        storage.initialize(constable);
        self.top_level_call(ICastle::createProtectedFunctionsCall {
            contract_address: constable,
            function_selectors: vec![
                IConstable::appointBankerCall::SELECTOR.into(),
                IConstable::appointFactorCall::SELECTOR.into(),
                IConstable::appointGuildmasterCall::SELECTOR.into(),
                IConstable::appointScribeCall::SELECTOR.into(),
                IConstable::appointWorksmanCall::SELECTOR.into(),
                IConstable::appendClerkChamberCall::SELECTOR.into(),
            ],
            required_role: CASTLE_ADMIN_ROLE.into(),
        })?;
        self.top_level_call(ICastle::createPublicFunctionsCall {
            contract_address: constable,
            function_selectors: vec![
                IConstable::getIssuerRoleCall::SELECTOR.into(),
                IConstable::getKeeperRoleCall::SELECTOR.into(),
                IConstable::getVendorRoleCall::SELECTOR.into(),
            ],
        })?;
        Ok(())
    }

    pub fn appoint_banker(&mut self, banker: Address) -> Result<(), Vec<u8>> {
        log_msg!("Appointing banker {}", banker);
        let storage = Keep::storage();
        if storage.constable.get().is_zero() {
            Err(b"Constable was not appointed")?;
        }
        self.top_level_call(ICastle::createProtectedFunctionsCall {
            contract_address: banker,
            function_selectors: vec![
                IBanker::submitAssetsCall::SELECTOR.into(),
                IBanker::submitMarginCall::SELECTOR.into(),
                IBanker::submitSupplyCall::SELECTOR.into(),
            ],
            required_role: CASTLE_VENDOR_ROLE.into(),
        })?;
        self.top_level_call(ICastle::createPublicFunctionsCall {
            contract_address: banker,
            function_selectors: vec![
                IBanker::getVendorAssetsCall::SELECTOR.into(),
                IBanker::getVendorMarginCall::SELECTOR.into(),
                IBanker::getVendorSupplyCall::SELECTOR.into(),
                IBanker::getVendorDemandCall::SELECTOR.into(),
                IBanker::getVendorDeltaCall::SELECTOR.into(),
            ],
        })?;
        Ok(())
    }

    pub fn appoint_factor(&mut self, factor: Address) -> Result<(), Vec<u8>> {
        log_msg!("Appointing factor {}", factor);
        let storage = Keep::storage();
        if storage.constable.get().is_zero() {
            Err(b"Constable was not appointed")?;
        }
        self.top_level_call(ICastle::createProtectedFunctionsCall {
            contract_address: factor,
            function_selectors: vec![IFactor::submitMarketDataCall::SELECTOR.into()],
            required_role: CASTLE_VENDOR_ROLE.into(),
        })?;
        // self.top_level_call(ICastle::createProtectedFunctionsCall {
        //     contract_address: factor,
        //     function_selectors: vec![
        //         IFactor::submitRebalanceOrderCall::SELECTOR.into(),
        //     ],
        //     required_role: CASTLE_ISSUER_ROLE.into(),
        // })?;
        self.top_level_call(ICastle::createProtectedFunctionsCall {
            contract_address: factor,
            function_selectors: vec![
                IFactor::updateIndexQuoteCall::SELECTOR.into(),
                IFactor::updateMultipleIndexQuotesCall::SELECTOR.into(),
                IFactor::submitBuyOrderCall::SELECTOR.into(),
                IFactor::submitSellOrderCall::SELECTOR.into(),
            ],
            required_role: CASTLE_KEEPER_ROLE.into(),
        })?;
        self.top_level_call(ICastle::createPublicFunctionsCall {
            contract_address: factor,
            function_selectors: vec![
                IFactor::submitTransferCall::SELECTOR.into(),
                IFactor::submitTransferFromCall::SELECTOR.into(),
                IFactor::approveTransferFromCall::SELECTOR.into(),
                IFactor::getTransferAllowanceCall::SELECTOR.into(),
                IFactor::getMarketDataCall::SELECTOR.into(),
                IFactor::getIndexAssetsCall::SELECTOR.into(),
                IFactor::getIndexWeightsCall::SELECTOR.into(),
                IFactor::getIndexQuoteCall::SELECTOR.into(),
                IFactor::getTraderOrderCall::SELECTOR.into(),
                IFactor::getTraderCountCall::SELECTOR.into(),
                IFactor::getTraderOrderAtCall::SELECTOR.into(),
                IFactor::getVendorOrderCall::SELECTOR.into(),
                IFactor::getVendorCountCall::SELECTOR.into(),
                IFactor::getVendorOrderAtCall::SELECTOR.into(),
                IFactor::getTotalOrderCall::SELECTOR.into(),
            ],
        })?;
        Ok(())
    }

    pub fn appoint_guildmaster(&mut self, guildmaster: Address) -> Result<(), Vec<u8>> {
        log_msg!("Appointing guildmaster {}", guildmaster);
        let storage = Keep::storage();
        if storage.constable.get().is_zero() {
            Err(b"Constable was not appointed")?;
        }
        self.top_level_call(ICastle::createProtectedFunctionsCall {
            contract_address: guildmaster,
            function_selectors: vec![
                IGuildmaster::submitIndexCall::SELECTOR.into(),
                IGuildmaster::submitVoteCall::SELECTOR.into(),
            ],
            required_role: CASTLE_ISSUER_ROLE.into(),
        })?;
        Ok(())
    }

    pub fn appoint_scribe(&mut self, scribe: Address) -> Result<(), Vec<u8>> {
        log_msg!("Appointing scribe {}", scribe);
        let storage = Keep::storage();
        if storage.constable.get().is_zero() {
            Err(b"Constable was not appointed")?;
        }
        self.inner_call(scribe, IScribe::acceptAppointmentCall { scribe })?;
        self.top_level_call(ICastle::createProtectedFunctionsCall {
            contract_address: scribe,
            function_selectors: vec![IScribe::verifySignatureCall::SELECTOR.into()],
            required_role: CASTLE_ISSUER_ROLE.into(),
        })?;
        Ok(())
    }

    pub fn appoint_worksman(&mut self, worksman: Address) -> Result<(), Vec<u8>> {
        log_msg!("Appointing worksman {}", worksman);
        let storage = Keep::storage();
        if storage.constable.get().is_zero() {
            Err(b"Constable was not appointed")?;
        }
        self.inner_call(worksman, IWorksman::acceptAppointmentCall { worksman })?;
        self.top_level_call(ICastle::createProtectedFunctionsCall {
            contract_address: worksman,
            function_selectors: vec![IWorksman::buildVaultCall::SELECTOR.into()],
            required_role: CASTLE_ISSUER_ROLE.into(),
        })?;
        Ok(())
    }

    pub fn append_clerk_chamber(&mut self, gate_to_clerk_chamber: Address) -> Result<(), Vec<u8>> {
        log_msg!("Appending clerk {}", gate_to_clerk_chamber);
        let mut storage = Keep::storage();
        if storage.constable.get().is_zero() {
            Err(b"Constable was not appointed")?;
        }
        if !storage.clerk_chamber.get_gate_address().is_zero() {
            Err(b"Clerk already cast")?;
        }
        storage.clerk_chamber.initialize(gate_to_clerk_chamber);
        Ok(())
    }

    pub fn get_issuer_role(&self) -> B256 {
        CASTLE_ISSUER_ROLE.into()
    }

    pub fn get_vendor_role(&self) -> B256 {
        CASTLE_VENDOR_ROLE.into()
    }

    pub fn get_keeper_role(&self) -> B256 {
        CASTLE_KEEPER_ROLE.into()
    }
}
