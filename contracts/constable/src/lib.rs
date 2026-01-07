// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, B256, U32};
use alloy_sol_types::SolCall;
use common::log_msg;
use common_contracts::{
    contracts::{calls::InnerCall, castle::CASTLE_ADMIN_ROLE, clerk::ClerkStorage, keep::{KEEP_VERSION_NUMBER, Keep}},
    interfaces::{
        banker::IBanker, castle::ICastle, clerk::IClerk, constable::IConstable, factor::IFactor, guildmaster::IGuildmaster, scribe::IScribe, worksman::IWorksman
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

pub const CASTLE_VAULT_ROLE: [u8; 32] = keccak_const::Keccak256::new()
    .update(b"Castle.VAULT_ROLE")
    .finalize();

pub const CASTLE_MAINTAINER_ROLE: [u8; 32] = keccak_const::Keccak256::new()
    .update(b"Castle.MAINTAINER_ROLE")
    .finalize();

#[storage]
#[entrypoint]
pub struct Constable;

#[public]
impl Constable {
    pub fn accept_appointment(&mut self, constable: Address) -> Result<(), Vec<u8>> {
        if constable.is_zero() {
            Err(b"Address cannot be zero")?;
        }
        let mut storage = Keep::storage();

        log_msg!("Appointing Constable {}", constable);
        storage.set_version()?;

        self.top_level_call(ICastle::createProtectedFunctionsCall {
            contract_address: constable,
            function_selectors: vec![
                IConstable::appointBankerCall::SELECTOR.into(),
                IConstable::appointFactorCall::SELECTOR.into(),
                IConstable::appointGuildmasterCall::SELECTOR.into(),
                IConstable::appointScribeCall::SELECTOR.into(),
                IConstable::appointWorksmanCall::SELECTOR.into(),
                IConstable::appointClerkCall::SELECTOR.into(),
            ],
            required_role: CASTLE_ADMIN_ROLE.into(),
        })?;

        self.top_level_call(ICastle::createPublicFunctionsCall {
            contract_address: constable,
            function_selectors: vec![
                IConstable::getIssuerRoleCall::SELECTOR.into(),
                IConstable::getKeeperRoleCall::SELECTOR.into(),
                IConstable::getVendorRoleCall::SELECTOR.into(),
                IConstable::getVaultRoleCall::SELECTOR.into(),
                IConstable::getMaintainerRoleCall::SELECTOR.into(),
                IConstable::getVersionCall::SELECTOR.into(),
            ],
        })?;

        Ok(())
    }

    pub fn appoint_banker(&mut self, banker: Address) -> Result<(), Vec<u8>> {
        if banker.is_zero() {
            Err(b"Address cannot be zero")?;
        }
        let storage = Keep::storage();
        storage.check_version()?;

        log_msg!("Appointing banker {}", banker);

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
        if factor.is_zero() {
            Err(b"Address cannot be zero")?;
        }
        let storage = Keep::storage();
        storage.check_version()?;

        log_msg!("Appointing factor {}", factor);

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
                IFactor::submitBuyOrderCall::SELECTOR.into(),
                IFactor::submitSellOrderCall::SELECTOR.into(),
            ],
            required_role: CASTLE_KEEPER_ROLE.into(),
        })?;

        self.top_level_call(ICastle::createProtectedFunctionsCall {
            contract_address: factor,
            function_selectors: vec![IFactor::submitTransferCall::SELECTOR.into()],
            required_role: CASTLE_VAULT_ROLE.into(),
        })?;

        self.top_level_call(ICastle::createPublicFunctionsCall {
            contract_address: factor,
            function_selectors: vec![
                IFactor::getMarketDataCall::SELECTOR.into(),
                IFactor::getIndexAssetsCountCall::SELECTOR.into(),
                IFactor::getIndexAssetsCall::SELECTOR.into(),
                IFactor::getIndexWeightsCall::SELECTOR.into(),
                IFactor::getIndexQuoteCall::SELECTOR.into(),
                IFactor::getTraderOrderCall::SELECTOR.into(),
                IFactor::getTraderCountCall::SELECTOR.into(),
                IFactor::getTraderAtCall::SELECTOR.into(),
                IFactor::getVendorOrderCall::SELECTOR.into(),
                IFactor::getVendorCountCall::SELECTOR.into(),
                IFactor::getVendorAtCall::SELECTOR.into(),
                IFactor::getTotalOrderCall::SELECTOR.into(),
            ],
        })?;
        Ok(())
    }

    pub fn appoint_guildmaster(&mut self, guildmaster: Address) -> Result<(), Vec<u8>> {
        if guildmaster.is_zero() {
            Err(b"Address cannot be zero")?;
        }
        let storage = Keep::storage();
        storage.check_version()?;

        log_msg!("Appointing guildmaster {}", guildmaster);

        self.top_level_call(ICastle::createProtectedFunctionsCall {
            contract_address: guildmaster,
            function_selectors: vec![
                IGuildmaster::submitIndexCall::SELECTOR.into(),
                IGuildmaster::submitVoteCall::SELECTOR.into(),
            ],
            required_role: CASTLE_ISSUER_ROLE.into(),
        })?;
        
        self.top_level_call(ICastle::createProtectedFunctionsCall {
            contract_address: guildmaster,
            function_selectors: vec![
                IGuildmaster::updateIndexQuoteCall::SELECTOR.into(),
                IGuildmaster::updateMultipleIndexQuotesCall::SELECTOR.into(),
            ],
            required_role: CASTLE_KEEPER_ROLE.into(),
        })?;

        Ok(())
    }

    pub fn appoint_scribe(&mut self, scribe: Address) -> Result<(), Vec<u8>> {
        if scribe.is_zero() {
            Err(b"Address cannot be zero")?;
        }
        let mut storage = Keep::storage();
        storage.check_version()?;

        log_msg!("Appointing scribe {}", scribe);
        storage.scribe.set(scribe);

        self.top_level_call(ICastle::createProtectedFunctionsCall {
            contract_address: scribe,
            function_selectors: vec![IScribe::verifySignatureCall::SELECTOR.into()],
            required_role: CASTLE_ISSUER_ROLE.into(),
        })?;

        Ok(())
    }

    pub fn appoint_worksman(&mut self, worksman: Address) -> Result<(), Vec<u8>> {
        if worksman.is_zero() {
            Err(b"Address cannot be zero")?;
        }
        let mut storage = Keep::storage();
        storage.check_version()?;
        
        log_msg!("Appointing worksman {}", worksman);
        storage.worksman.set(worksman);

        self.top_level_call(ICastle::createProtectedFunctionsCall {
            contract_address: worksman,
            function_selectors: vec![IWorksman::buildVaultCall::SELECTOR.into()],
            required_role: CASTLE_ISSUER_ROLE.into(),
        })?;

        self.top_level_call(ICastle::createProtectedFunctionsCall {
            contract_address: worksman,
            function_selectors: vec![IWorksman::addVaultCall::SELECTOR.into()],
            required_role: CASTLE_ADMIN_ROLE.into(),
        })?;

        Ok(())
    }

    pub fn appoint_clerk(&mut self, clerk: Address) -> Result<(), Vec<u8>> {
        if clerk.is_zero() {
            Err(b"Address cannot be zero")?;
        }
        let mut storage = Keep::storage();
        storage.check_version()?;

        log_msg!("Appointing clerk {}", clerk);
        storage.clerk.set(clerk);

        let mut clerk_storage = ClerkStorage::storage();
        if !clerk_storage.is_constructed() {
            clerk_storage.constructor()?;
        }
        
        self.top_level_call(ICastle::createProtectedFunctionsCall {
            contract_address: clerk,
            function_selectors: vec![IClerk::fetchVectorCall::SELECTOR.into()],
            required_role: CASTLE_MAINTAINER_ROLE.into(),
        })?;

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

    pub fn get_vault_role(&self) -> B256 {
        CASTLE_VAULT_ROLE.into()
    }
    
    pub fn get_maintainer_role(&self) -> B256 {
        CASTLE_MAINTAINER_ROLE.into()
    }
    
    pub fn get_version(&self) -> U32 {
        KEEP_VERSION_NUMBER
    }
}
