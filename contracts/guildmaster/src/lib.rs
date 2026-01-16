// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::{string::String, vec::Vec};

use alloy_primitives::{Address, U128};
use common::{labels::Labels, vector::Vector};
use common_contracts::{
    contracts::{
        calls::InnerCall,
        castle::{CastleStorage, CASTLE_KEEPER_ROLE, CASTLE_VAULT_ROLE},
        clerk::ClerkStorage,
        keep::{Keep, VAULT_STATUS_APPROVED, VAULT_STATUS_NEW, VAULT_STATUS_REJECTED},
        keep_calls::KeepCalls,
    },
    interfaces::{
        guildmaster::IGuildmaster, vault::IVault, vault_native::IVaultNative,
    },
};
use stylus_sdk::{abi::Bytes, prelude::*, stylus_core};

#[storage]
#[entrypoint]
pub struct Guildmaster;

#[public]
impl Guildmaster {
    /// Submit new Index
    ///
    /// Deploys Vault contract in inactive state. Needs to be voted to activate.
    ///
    pub fn submit_index(
        &mut self,
        vendor_id: U128,
        index_id: U128,
        name: String,
        symbol: String,
        description: String,
        methodology: String,
        initial_price: U128,
        curator: Address,
        custody: String,
        operators: Vec<Address>,
        collateral_custody: Address,
        collateral_asset: Address,
        max_order_size: U128,
    ) -> Result<Address, Vec<u8>> {
        if vendor_id.is_zero() {
            Err(b"Vendor ID cannot be zero")?;
        }
        if index_id.is_zero() {
            Err(b"Index ID cannot be zero")?;
        }

        let mut storage = Keep::storage();
        storage.check_version()?;

        let mut vault = storage.vaults.setter(index_id);
        vault.only_uninitialized()?;

        let worksman = storage.worksman.get();
        let gate_to_vault = self
            .build_vault(worksman)
            .map_err(|_| b"Failed to build vault")?;

        vault.gate_to_vault.set(gate_to_vault);
        vault.status.set(VAULT_STATUS_NEW);

        self.external_call(
            gate_to_vault,
            IVault::configureVaultCall {
                index_id: index_id.to(),
                name: name.clone(),
                symbol: symbol.clone(),
                description,
                methodology,
                initial_price: initial_price.to(),
                curator,
                custody,
            },
        )
        .map_err(|_| b"Failed to configure vault")?;

        self.external_call(
            gate_to_vault,
            IVaultNative::configureRequestsCall {
                vendor_id: vendor_id.to(),
                custody: collateral_custody,
                asset: collateral_asset,
                max_order_size: max_order_size.to(),
            },
        )
        .map_err(|_| b"Failed to configure requests")?;

        self.external_call(
            gate_to_vault,
            IVault::addCustodiansCall {
                accounts: operators,
            },
        )
        .map_err(|_| b"Failed to add operators")?;

        let mut castle_storage = CastleStorage::storage();
        let acl = castle_storage.get_acl_mut();

        acl.set_role(gate_to_vault, CASTLE_KEEPER_ROLE.into())
            .map_err(|_| b"Failed to set vault role")?;

        acl.set_role(gate_to_vault, CASTLE_VAULT_ROLE.into())
            .map_err(|_| b"Failed to set vault role")?;

        stylus_core::log(
            self.vm(),
            IGuildmaster::IndexCreated {
                index_id: index_id.to(),
                name,
                symbol,
                vault: gate_to_vault,
            },
        );

        Ok(gate_to_vault)
    }

    pub fn begin_edit_index(&mut self, index_id: U128) -> Result<(), Vec<u8>> {
        if index_id.is_zero() {
            Err(b"Index ID cannot be zero")?;
        }

        let mut storage = Keep::storage();
        let sender = self.attendee();
        storage.check_version()?;

        let vault = storage.vaults.setter(index_id);
        vault.only_initialized()?;

        self.external_call(
            vault.gate_to_vault.get(),
            IVault::transferOwnershipCall { new_owner: sender },
        )?;

        stylus_core::log(
            self.vm(),
            IGuildmaster::BeginEditIndex {
                index_id: index_id.to(),
                sender,
            },
        );

        Ok(())
    }

    pub fn finish_edit_index(&mut self, index_id: U128) -> Result<(), Vec<u8>> {
        if index_id.is_zero() {
            Err(b"Index ID cannot be zero")?;
        }

        let mut storage = Keep::storage();
        let sender = self.attendee();
        storage.check_version()?;

        let vault = storage.vaults.setter(index_id);
        vault.only_initialized()?;

        let IVault::ownerReturn { _0: owner } =
            self.static_call_ret(vault.gate_to_vault.get(), IVault::ownerCall {})?;

        if owner != self.top_level() {
            Err(b"Vault ownership must be returned")?;
        }

        stylus_core::log(
            self.vm(),
            IGuildmaster::FinishEditIndex {
                index_id: index_id.to(),
                sender,
            },
        );

        Ok(())
    }

    pub fn submit_asset_weights(
        &mut self,
        index_id: U128,
        asset_names: Bytes,
        asset_weights: Bytes,
    ) -> Result<(), Vec<u8>> {
        if index_id.is_zero() {
            Err(b"Index ID cannot be zero")?;
        }
        let num_assets =
            Labels::len_from_vec(&asset_names).ok_or_else(|| b"Invalid Asset Names")?;

        if num_assets
            != Vector::len_from_vec(&asset_weights).ok_or_else(|| b"Invalid Asset Weights")?
        {
            Err(b"Asset Names and Asset Weights are not aligned")?;
        }

        let mut storage = Keep::storage();
        let sender = self.attendee();
        storage.check_version()?;

        let mut vault = storage.vaults.setter(index_id);
        vault.only_initialized()?;

        if !vault.assets.get().is_zero() {
            Err(b"Asset Weights already set")?;
        }

        let mut clerk_storage = ClerkStorage::storage();
        let asset_names_id = clerk_storage.next_vector();
        let asset_weights_id = clerk_storage.next_vector();

        clerk_storage.store_bytes(asset_names_id, asset_names);
        clerk_storage.store_bytes(asset_weights_id, asset_weights);

        vault.assets.set(asset_names_id);
        vault.weights.set(asset_weights_id);

        stylus_core::log(
            self.vm(),
            IGuildmaster::IndexWeightsUpdated {
                index_id: index_id.to(),
                sender,
            },
        );

        Ok(())
    }

    /// Submit a vote for an Index
    ///
    /// Once enough votes, Vault contract is activated.
    ///
    pub fn submit_vote(&mut self, index_id: U128, vote: Bytes) -> Result<(), Vec<u8>> {
        if index_id.is_zero() {
            Err(b"Index ID cannot be zero")?;
        }

        let mut storage = Keep::storage();
        let sender = self.attendee();
        storage.check_version()?;

        let mut vault = storage.vaults.setter(index_id);
        vault.only_unvoted()?;

        let scribe = storage.scribe.get();
        let verfication_result = self.verify_signature(scribe, vote.0)?;

        if verfication_result {
            vault.status.set(VAULT_STATUS_APPROVED);
        } else {
            vault.status.set(VAULT_STATUS_REJECTED);
        }

        stylus_core::log(
            self.vm(),
            IGuildmaster::IndexVoteUpdated {
                index_id: index_id.to(),
                sender,
            },
        );

        Ok(())
    }
}
