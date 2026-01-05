use alloc::{vec, vec::Vec};

use alloy_primitives::{uint, Address, U128, U256};
use stylus_sdk::{
    keccak_const,
    prelude::*,
    storage::{StorageAddress, StorageMap, StorageU128, StorageVec},
};

use crate::contracts::storage::StorageSlot;

pub const KEEP_STORAGE_SLOT: U256 = {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"Keep.STORAGE_SLOT")
        .finalize();
    U256::from_be_bytes(HASH).wrapping_sub(uint!(1_U256))
};

#[storage]
pub struct Allowance {
    from_account: StorageMap<Address, StorageU128>,
}

impl Allowance {
    pub fn allowance(&self, spender: Address) -> u128 {
        self.from_account.get(spender).to()
    }

    pub fn approve(&mut self, spender: Address, value: u128) -> Result<bool, Vec<u8>> {
        if spender.is_zero() {
            Err(b"Invalid Spender")?;
        }
        let mut allowance = self.from_account.setter(spender);
        allowance.set(U128::from(value));
        Ok(true)
    }

    pub fn spend_allowance(&mut self, spender: Address, value: u128) -> Result<(), Vec<u8>> {
        if spender.is_zero() {
            Err(b"Invalid Spender")?;
        }
        let mut allowance = self.from_account.setter(spender);
        let current = allowance.get();
        let remain = current
            .checked_sub(U128::from(value))
            .ok_or_else(|| b"Insufficient Allowance")?;
        allowance.set(remain);
        Ok(())
    }
}

#[storage]
pub struct VaultRecords {
    pub allowances: StorageMap<Address, Allowance>,
}

#[storage]
pub struct Vault {
    pub gate_to_vault: StorageAddress,
    pub records: VaultRecords,

    // Index definition
    pub assets: StorageU128,  // Labels  = [u128; num_assets]
    pub weights: StorageU128, // Vector  = [Amount; num_assets]

    // Rebalance vectors
    pub rebalance_weights_long: StorageU128, // Vector = [Amount; num_assets]
    pub rebalance_weights_short: StorageU128, // Vector = [Amount; num_assets]

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
pub struct ClerkChamber {
    gate_to_clerk_chamber: StorageAddress,
    last_vector: StorageU128,
}

impl ClerkChamber {
    pub const SCRATCH_1: U128 = uint!(1_U128);
    pub const SCRATCH_2: U128 = uint!(2_U128);
    pub const SCRATCH_3: U128 = uint!(3_U128);
    pub const SCRATCH_4: U128 = uint!(4_U128);

    pub const FIRST_DYNAMIC_ID: U128 = uint!(100_U128);

    pub fn initialize(&mut self, gate_to_clerk_chamber: Address) {
        self.gate_to_clerk_chamber.set(gate_to_clerk_chamber);
        self.last_vector.set(uint!(Self::FIRST_DYNAMIC_ID));
    }

    pub fn next_vector(&mut self) -> U128 {
        let value = self.last_vector.get() + U128::ONE;
        self.last_vector.set(value);
        value
    }

    pub fn get_gate_address(&self) -> Address {
        self.gate_to_clerk_chamber.get()
    }
}

#[storage]
pub struct Keep {
    pub vaults: StorageMap<U128, Vault>,
    pub accounts: StorageMap<U128, Account>,
    pub clerk_chamber: ClerkChamber,
    pub constable: StorageAddress,
    pub worksman: StorageAddress,
    pub scribe: StorageAddress,
    pub solve_quadratic_bid_id: StorageU128,
    pub solve_quadratic_ask_id: StorageU128,
}

impl Keep {
    pub fn storage() -> Keep {
        StorageSlot::get_slot::<Keep>(KEEP_STORAGE_SLOT)
    }

    pub fn initialize(&mut self, constable: Address) {
        self.constable.set(constable);
    }
}
