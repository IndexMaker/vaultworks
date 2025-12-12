use alloc::{vec, vec::Vec};

use alloy_primitives::{uint, Address, U128, U256};
use stylus_sdk::{
    keccak_const,
    prelude::*,
    storage::{StorageAddress, StorageMap, StorageU128},
};

use crate::storage::StorageSlot;

pub const KEEP_STORAGE_SLOT: U256 = {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"Keep.STORAGE_SLOT")
        .finalize();
    U256::from_be_bytes(HASH).wrapping_sub(uint!(1_U256))
};

#[storage]
pub struct Vault {
    pub gate_to_vault: StorageAddress,
    pub assets: StorageU128,  // Labels  = [u128; num_assets]
    pub weights: StorageU128, // Vector  = [Amount; num_assets]
    pub quote: StorageU128,   // Vector  = [Capacity, Price, Slope]
    pub orders: StorageMap<Address, StorageU128>, // Mapping = {User Address => Vector = [USDC Remaining, USDC Spent, ITP Minted]}
    pub queue: StorageU128,                       // Labels  = [u128; num_orders]
}

#[storage]
pub struct Account {
    owner: StorageAddress,
    // TODO: These will be very long vectors, e.g. 2M components.
    // We will optimise Granary and Clerk to provide partial load/store
    // and we'll store chunks in mapping.
    pub assets: StorageU128,       // Vector = [Name; num_assets]
    pub margin: StorageU128,       // Vector = [Margin; num_assets]
    // Delta = Suppy - Demand
    pub supply_long: StorageU128,  // Vector = [+Supply; num_assets]
    pub supply_short: StorageU128, // Vector = [-Supply; num_assets]
    pub demand_long: StorageU128,  // Vector = [+Demand; num_assets]
    pub demand_short: StorageU128, // Vector = [-Demand; num_assets]
    pub delta_long: StorageU128,   // Vector = [+Delta; num_assets]
    pub delta_short: StorageU128,  // Vector = [-Delta; num_assets]
    // Market Data
    pub liquidity: StorageU128,    // Vector = [Liquidity; num_assets]
    pub prices: StorageU128,       // Vector = [Price; num_assets]
    pub slopes: StorageU128,       // Vector = [Slope; num_assets]
}

impl Account {
    pub fn is_owner(&self, address: Address) -> bool {
        self.owner.get() == address
    } 

    pub fn set_only_owner(&mut self, address: Address) -> Result<(), Vec<u8>> {
        let owner = self.owner.get();
        if owner.is_zero() {
            self.owner.set(address);
        }
        else if owner != address {
            Err(b"Unauthorized access")?;
        }
        Ok(())
    }
}

#[storage]
pub struct Granary {
    gate_to_granary: StorageAddress,
    last_vector: StorageU128,
}

impl Granary {
    pub const SCRATCH_1: U128 = uint!(1_U128);
    pub const SCRATCH_2: U128 = uint!(2_U128);
    pub const SCRATCH_3: U128 = uint!(3_U128);
    pub const SCRATCH_4: U128 = uint!(4_U128);

    pub const FIRST_DYNAMIC_ID: U128 = uint!(100_U128);

    pub fn initialize(&mut self, gate_to_granary: Address) {
        self.gate_to_granary.set(gate_to_granary);
        self.last_vector.set(uint!(Self::FIRST_DYNAMIC_ID));
    }

    pub fn next_vector(&mut self) -> U128 {
        let value = self.last_vector.get() + U128::ONE;
        self.last_vector.set(value);
        value
    }

    pub fn get_granary_address(&self) -> Address {
        self.gate_to_granary.get()
    }
}

#[storage]
pub struct Keep {
    pub vaults: StorageMap<U128, Vault>,
    pub accounts: StorageMap<U128, Account>,
    pub granary: Granary,
    pub castle: StorageAddress,
    pub constable: StorageAddress,
    pub worksman: StorageAddress,
}

impl Keep {
    pub fn storage() -> Keep {
        StorageSlot::get_slot::<Keep>(KEEP_STORAGE_SLOT)
    }

    pub fn initialize(&mut self, castle: Address, constable: Address) {
        self.castle.set(castle);
        self.constable.set(constable);
    }
}
