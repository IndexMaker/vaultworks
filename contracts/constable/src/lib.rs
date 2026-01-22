// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{aliases::B32, Address, B256, U32};
use alloy_sol_types::{SolCall, SolEvent};
use common::log_msg;
use common_contracts::{
    contracts::{
        acl::AccessControlList,
        castle::{
            CastleStorage, CASTLE_ADMIN_ROLE, CASTLE_ISSUER_ROLE, CASTLE_KEEPER_ROLE,
            CASTLE_MAINTAINER_ROLE, CASTLE_VAULT_ROLE, CASTLE_VENDOR_ROLE,
        },
        clerk::ClerkStorage,
        keep::{Keep, KEEP_VERSION_NUMBER},
    },
    interfaces::{
        alchemist::IAlchemist, banker::IBanker, castle::ICastle, constable::IConstable,
        factor::IFactor, guildmaster::IGuildmaster, scribe::IScribe, steward::ISteward,
        worksman::IWorksman,
    },
};
use stylus_sdk::{prelude::*, stylus_core};

#[storage]
#[entrypoint]
pub struct Constable;

impl Constable {
    fn _publish_event<T>(&self, event: T)
    where
        T: SolEvent,
    {
        stylus_core::log(self.vm(), event);
    }

    fn _attendee(&self) -> Address {
        self.vm().msg_sender()
    }

    fn _only_admin(&self, acl: &AccessControlList) -> Result<(), Vec<u8>> {
        acl.only_role(CASTLE_ADMIN_ROLE.into(), self._attendee())?;
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
    /// Associate function selectors with delegate setting **public** access.
    ///
    /// Parameters
    /// ----------
    /// - contract_address: An address of the contract implementing the functions.
    /// - fun_selectors: A list of function selectors (first 4 bytes of EVM ABI call encoding).
    ///
    /// Everyone will be able to access listed functions.
    ///
    fn _create_public_functions(
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
    fn _create_protected_functions(
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
    fn _remove_functions(&mut self, function_selectors: Vec<B32>) -> Result<(), Vec<u8>> {
        self._check_functions(&function_selectors)?;

        let mut storage = CastleStorage::storage();
        let acl = storage.get_acl_mut();

        self._only_admin(acl)?;
        storage.set_functions(None, None, &function_selectors);

        self._publish_event(ICastle::FunctionsRemoved { function_selectors });
        Ok(())
    }
}

#[public]
impl Constable {
    pub fn accept_appointment(&mut self, constable: Address) -> Result<(), Vec<u8>> {
        if constable.is_zero() {
            Err(b"Address cannot be zero")?;
        }
        let mut storage = Keep::storage();

        log_msg!("Appointing Constable {}", constable);
        storage.set_version()?;

        self._create_protected_functions(
            constable,
            vec![
                IConstable::appointBankerCall::SELECTOR.into(),
                IConstable::appointFactorCall::SELECTOR.into(),
                IConstable::appointGuildmasterCall::SELECTOR.into(),
                IConstable::appointScribeCall::SELECTOR.into(),
                IConstable::appointWorksmanCall::SELECTOR.into(),
                IConstable::appointClerkCall::SELECTOR.into(),
                IConstable::appointStewardCall::SELECTOR.into(),
            ],
            CASTLE_ADMIN_ROLE.into(),
        )?;

        self._create_public_functions(
            constable,
            vec![
                IConstable::getIssuerRoleCall::SELECTOR.into(),
                IConstable::getKeeperRoleCall::SELECTOR.into(),
                IConstable::getVendorRoleCall::SELECTOR.into(),
                IConstable::getVaultRoleCall::SELECTOR.into(),
                IConstable::getMaintainerRoleCall::SELECTOR.into(),
                IConstable::getVersionCall::SELECTOR.into(),
            ],
        )?;

        Ok(())
    }

    //
    // Castle's NPCs (Diamond Facets)
    //

    pub fn appoint_banker(&mut self, banker: Address) -> Result<(), Vec<u8>> {
        if banker.is_zero() {
            Err(b"Address cannot be zero")?;
        }
        let storage = Keep::storage();
        storage.check_version()?;

        log_msg!("Appointing banker {}", banker);

        self._create_protected_functions(
            banker,
            vec![
                IBanker::submitAssetsCall::SELECTOR.into(),
                IBanker::submitMarginCall::SELECTOR.into(),
                IBanker::submitSupplyCall::SELECTOR.into(),
                IBanker::submitMarketDataCall::SELECTOR.into(),
            ],
            CASTLE_VENDOR_ROLE.into(),
        )?;

        self._create_protected_functions(
            banker,
            vec![
                IBanker::updateIndexQuoteCall::SELECTOR.into(),
                IBanker::updateMultipleIndexQuotesCall::SELECTOR.into(),
            ],
            CASTLE_KEEPER_ROLE.into(),
        )?;

        Ok(())
    }

    pub fn appoint_factor(&mut self, factor: Address) -> Result<(), Vec<u8>> {
        if factor.is_zero() {
            Err(b"Address cannot be zero")?;
        }
        let storage = Keep::storage();
        storage.check_version()?;

        log_msg!("Appointing factor {}", factor);

        self._create_protected_functions(
            factor,
            vec![
                IFactor::submitBuyOrderCall::SELECTOR.into(),
                IFactor::submitSellOrderCall::SELECTOR.into(),
                IFactor::processPendingBuyOrderCall::SELECTOR.into(),
                IFactor::processPendingSellOrderCall::SELECTOR.into(),
                IFactor::executeBuyOrderCall::SELECTOR.into(),
                IFactor::executeSellOrderCall::SELECTOR.into(),
                IFactor::executeTransferCall::SELECTOR.into(),
            ],
            CASTLE_VAULT_ROLE.into(),
        )?;

        Ok(())
    }

    pub fn appoint_steward(&mut self, steward: Address) -> Result<(), Vec<u8>> {
        if steward.is_zero() {
            Err(b"Address cannot be zero")?;
        }
        let storage = Keep::storage();
        storage.check_version()?;

        log_msg!("Appointing steward {}", steward);

        self._create_public_functions(
            steward,
            vec![
                ISteward::getVaultCall::SELECTOR.into(),
                ISteward::getMarketDataCall::SELECTOR.into(),
                ISteward::getIndexAssetsCountCall::SELECTOR.into(),
                ISteward::getIndexAssetsCall::SELECTOR.into(),
                ISteward::getIndexWeightsCall::SELECTOR.into(),
                ISteward::getIndexQuoteCall::SELECTOR.into(),
                ISteward::getTraderOrderCall::SELECTOR.into(),
                ISteward::getTraderCountCall::SELECTOR.into(),
                ISteward::getTraderAtCall::SELECTOR.into(),
                ISteward::getVendorOrderCall::SELECTOR.into(),
                ISteward::getVendorCountCall::SELECTOR.into(),
                ISteward::getVendorAtCall::SELECTOR.into(),
                ISteward::getTotalOrderCall::SELECTOR.into(),
                ISteward::getVendorAssetsCall::SELECTOR.into(),
                ISteward::getVendorMarginCall::SELECTOR.into(),
                ISteward::getVendorSupplyCall::SELECTOR.into(),
                ISteward::getVendorDemandCall::SELECTOR.into(),
                ISteward::getVendorDeltaCall::SELECTOR.into(),
            ],
        )?;

        self._create_protected_functions(
            steward,
            vec![ISteward::fetchVectorCall::SELECTOR.into()],
            CASTLE_MAINTAINER_ROLE.into(),
        )?;

        Ok(())
    }

    pub fn appoint_guildmaster(&mut self, guildmaster: Address) -> Result<(), Vec<u8>> {
        if guildmaster.is_zero() {
            Err(b"Address cannot be zero")?;
        }
        let storage = Keep::storage();
        storage.check_version()?;

        log_msg!("Appointing guildmaster {}", guildmaster);

        self._create_protected_functions(
            guildmaster,
            vec![
                IGuildmaster::submitIndexCall::SELECTOR.into(),
                IGuildmaster::submitVoteCall::SELECTOR.into(),
            ],
            CASTLE_ISSUER_ROLE.into(),
        )?;

        self._create_protected_functions(
            guildmaster,
            vec![
                IGuildmaster::beginEditIndexCall::SELECTOR.into(),
                IGuildmaster::finishEditIndexCall::SELECTOR.into(),
            ],
            CASTLE_ADMIN_ROLE.into(),
        )?;

        Ok(())
    }

    pub fn appoint_alchemist(&mut self, alchemist: Address) -> Result<(), Vec<u8>> {
        if alchemist.is_zero() {
            Err(b"Address cannot be zero")?;
        }
        let storage = Keep::storage();
        storage.check_version()?;

        log_msg!("Appointing alchemist {}", alchemist);

        self._create_protected_functions(
            alchemist,
            vec![IAlchemist::submitAssetWeightsCall::SELECTOR.into()],
            CASTLE_ISSUER_ROLE.into(),
        )?;

        self._create_protected_functions(
            alchemist,
            vec![IAlchemist::processPendingRebalanceCall::SELECTOR.into()],
            CASTLE_KEEPER_ROLE.into(),
        )?;
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

        self._create_protected_functions(
            scribe,
            vec![IScribe::verifySignatureCall::SELECTOR.into()],
            CASTLE_ISSUER_ROLE.into(),
        )?;

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

        self._create_protected_functions(
            worksman,
            vec![IWorksman::buildVaultCall::SELECTOR.into()],
            CASTLE_ISSUER_ROLE.into(),
        )?;

        self._create_protected_functions(
            worksman,
            vec![IWorksman::setVaultPrototypeCall::SELECTOR.into()],
            CASTLE_ADMIN_ROLE.into(),
        )?;

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

        Ok(())
    }

    //
    // Roles
    //

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
