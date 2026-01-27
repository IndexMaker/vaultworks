use alloc::{vec, vec::Vec};

use alloy_primitives::{uint, Address, U128, U256, U32, U8};
use stylus_sdk::{
    keccak_const,
    prelude::*,
    storage::{StorageAddress, StorageBytes, StorageMap, StorageU8, StorageU32, StorageU128, StorageVec},
};

use crate::contracts::storage::StorageSlot;

pub const KEEP_VERSION_NUMBER: U32 = uint!(1_U32);

pub const KEEP_STORAGE_SLOT: U256 = {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"Keep.STORAGE_SLOT")
        .finalize();
    U256::from_be_bytes(HASH).wrapping_sub(uint!(1_U256))
};

pub const VAULT_STATUS_UNINITIALIZED: U8 = uint!(0_U8);
pub const VAULT_STATUS_NEW: U8 = uint!(1_U8);
pub const VAULT_STATUS_APPROVED: U8 = uint!(2_U8);
pub const VAULT_STATUS_REJECTED: U8 = uint!(3_U8);

#[storage]
pub struct Vault {
    pub status: StorageU8,
    pub gate_to_vault: StorageAddress,

    // Index definition
    pub assets: StorageU128,  // Labels  = [u128; num_assets]
    pub weights: StorageU128, // Vector  = [Amount; num_assets]

    // Rebalance vectors
    pub rebalance_assets: StorageU128, // Lablels = [u128l num_rebalance_assets]
    pub rebalance_weights_long: StorageU128, // Vector = [Amount; num_rebalance_assets]
    pub rebalance_weights_short: StorageU128, // Vector = [Amount; num_rebalance_assets]

    // Index pricing (TBD: could be mapping per vendor)
    pub vendor_quotes: StorageMap<U128, StorageU128>, // Mapping = { Vendor ID => Vector  = [Capacity, Price, Slope] }

    // Traders who founded that vault, or who redeemed the token
    pub traders: StorageVec<StorageAddress>, // List of addresses that trade this ITP token
    pub traders_bids: StorageMap<Address, StorageU128>, // Mapping = {User Address => Vector = [USDC Remaining, USDC Spent, ITP Minted]}
    pub traders_asks: StorageMap<Address, StorageU128>, // Mapping = {User Address => Vector = [ITP Remaining, ITP Burned, USDC Withdrawn]}

    // These are needed for ERC-4626 to know the share in total liquidity
    // {{

    // Stats across vendors
    pub vendors: StorageVec<StorageU128>, // List of vendor IDs that participated
    pub vendors_bids: StorageMap<U128, StorageU128>, // Mapping = {Vendor ID => Vector = [ITP Remaining, ITP Burned, USDC Withdrawn]}
    pub vendors_asks: StorageMap<U128, StorageU128>, // Mapping = {Vendor ID => Vector = [ITP Remaining, ITP Burned, USDC Withdrawn]}

    // Totals
    pub total_bid: StorageU128, // Vector = [USDC Remaining, USDC Spent, ITP Minted]
    pub total_ask: StorageU128, // Vector = [ITP Remaining, ITP Burned, USDC Withdrawn]

                                // }}
}

impl Vault {
    pub fn only_uninitialized(&self) -> Result<(), Vec<u8>> {
        match self.status.get() {
            VAULT_STATUS_UNINITIALIZED => Ok(()),
            VAULT_STATUS_NEW => Err(b"Vault already exists")?,
            VAULT_STATUS_REJECTED => Err(b"Vault voted (rejected)")?,
            VAULT_STATUS_APPROVED => Err(b"Vault voted (accepted)")?,
            _ => panic!("Unexpected vault status"),
        }
    }

    pub fn only_initialized(&self) -> Result<(), Vec<u8>> {
        match self.status.get() {
            VAULT_STATUS_UNINITIALIZED => Err(b"Vault does not exist")?,
            VAULT_STATUS_NEW => Ok(()),
            VAULT_STATUS_REJECTED => Ok(()),
            VAULT_STATUS_APPROVED => Ok(()),
            _ => panic!("Unexpected vault status"),
        }
    }

    pub fn only_unvoted(&self) -> Result<(), Vec<u8>> {
        match self.status.get() {
            VAULT_STATUS_UNINITIALIZED => Err(b"Vault does not exist")?,
            VAULT_STATUS_NEW => Ok(()),
            VAULT_STATUS_REJECTED => Err(b"Vault voted (rejected)")?,
            VAULT_STATUS_APPROVED => Err(b"Vault voted (accepted)")?,
            _ => panic!("Unexpected vault status"),
        }
    }

    pub fn only_tradeable(&self) -> Result<(), Vec<u8>> {
        match self.status.get() {
            VAULT_STATUS_UNINITIALIZED => Err(b"Vault does not exist")?,
            VAULT_STATUS_NEW => Err(b"Vault not voted")?,
            VAULT_STATUS_REJECTED => Err(b"Vault was rejected")?,
            VAULT_STATUS_APPROVED => Ok(()),
            _ => panic!("Unexpected vault status"),
        }
    }
}

#[storage]
pub struct Account {
    owner: StorageAddress,

    // TODO: These will be very long vectors, e.g. 2M components.
    // We will optimise Clerk and Abacus to provide partial load/store
    // and we'll store chunks in mapping.
    pub assets: StorageU128, // Vector = [Name; num_assets]
    pub margin: StorageU128, // Vector = [Margin; num_assets]

    // Delta = Suppy - Demand
    pub supply_long: StorageU128,  // Vector = [+Supply; num_assets]
    pub supply_short: StorageU128, // Vector = [-Supply; num_assets]
    pub demand_long: StorageU128,  // Vector = [+Demand; num_assets]
    pub demand_short: StorageU128, // Vector = [-Demand; num_assets]
    pub delta_long: StorageU128,   // Vector = [+Delta; num_assets]
    pub delta_short: StorageU128,  // Vector = [-Delta; num_assets]

    // Market Data
    pub liquidity: StorageU128, // Vector = [Liquidity; num_assets]
    pub prices: StorageU128,    // Vector = [Price; num_assets]
    pub slopes: StorageU128,    // Vector = [Slope; num_assets]
}

impl Account {
    pub fn is_owner(&self, address: Address) -> bool {
        self.owner.get() == address
    }

    pub fn only_owner(&self, address: Address) -> Result<(), Vec<u8>> {
        let owner = self.owner.get();
        if owner.is_zero() {
            Err(b"Owner not set")?;
        }
        if owner != address {
            Err(b"Unauthorized access")?;
        }
        Ok(())
    }

    pub fn set_owner(&mut self, address: Address) -> Result<(), Vec<u8>> {
        if self.has_owner() {
            Err(b"Owner already set")?;
        }
        self.owner.set(address);
        Ok(())
    }

    pub fn has_owner(&self) -> bool {
        !self.owner.get().is_zero()
    }
}

#[storage]
pub struct Keep {
    // Integrity Protection
    version: StorageU32,

    // Vaults & Accounts
    pub accounts: StorageMap<U128, Account>,
    pub vaults: StorageMap<U128, Vault>,

    // Stored Procedures
    pub solve_quadratic_bid_id: StorageU128,
    pub solve_quadratic_ask_id: StorageU128,
    
    // NPCs
    pub clerk: StorageAddress,
    pub scribe: StorageAddress,
    pub worksman: StorageAddress,
}

impl Keep {
    pub fn storage() -> Keep {
        StorageSlot::get_slot::<Keep>(KEEP_STORAGE_SLOT)
    }

    pub fn set_version(&mut self) -> Result<(), Vec<u8>> {
        if self.version.get() > KEEP_VERSION_NUMBER {
            Err(b"Keep downgrade prohibited")?;
        }
        self.version.set(KEEP_VERSION_NUMBER);
        Ok(())
    }

    pub fn check_version(&self) -> Result<(), Vec<u8>> {
        if self.version.get() != KEEP_VERSION_NUMBER {
            Err(b"Keep version incorrect")?;
        }
        Ok(())
    }
}
