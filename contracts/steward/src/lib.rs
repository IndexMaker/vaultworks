// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U128};
use common::vector::Vector;
use common_contracts::contracts::{clerk::ClerkStorage, formulas::Order, keep::Keep};
use stylus_sdk::{abi::Bytes, prelude::*};
use vector_macros::amount_vec;

#[storage]
#[entrypoint]
pub struct Steward;

#[public]
impl Steward {
    //
    // Query methods (Guildmaster)
    //

    pub fn get_vault(&self, index_id: U128) -> Result<Address, Vec<u8>> {
        let storage = Keep::storage();
        storage.check_version()?;

        let vault = storage.vaults.get(index_id);
        Ok(vault.gate_to_vault.get())
    }
    
    //
    // Query methods (Factor)
    //

    pub fn get_market_data(&self, vendor_id: U128) -> Result<Vec<Bytes>, Vec<u8>> {
        let storage = Keep::storage();
        storage.check_version()?;

        let clerk_storage = ClerkStorage::storage();
        let account = storage.accounts.get(vendor_id);

        let liquidity = clerk_storage
            .fetch_bytes(account.liquidity.get())
            .ok_or_else(|| b"Liquidity not set")?;

        let prices = clerk_storage
            .fetch_bytes(account.prices.get())
            .ok_or_else(|| b"Prices not set")?;

        let slopes = clerk_storage
            .fetch_bytes(account.slopes.get())
            .ok_or_else(|| b"Slopes not set")?;

        Ok(vec![liquidity.into(), prices.into(), slopes.into()])
    }

    pub fn get_index_assets_count(&self, index_id: U128) -> Result<U128, Vec<u8>> {
        let storage = Keep::storage();
        storage.check_version()?;

        let clerk_storage = ClerkStorage::storage();
        let vault = storage.vaults.get(index_id);

        let data = clerk_storage.len_vector(vault.assets.get());

        Ok(U128::from(data))
    }

    pub fn get_index_assets(&self, index_id: U128) -> Result<Bytes, Vec<u8>> {
        let storage = Keep::storage();
        storage.check_version()?;

        let clerk_storage = ClerkStorage::storage();
        let vault = storage.vaults.get(index_id);

        let data = clerk_storage
            .fetch_bytes(vault.assets.get())
            .ok_or_else(|| b"Assets not set")?;

        Ok(data.into())
    }

    pub fn get_index_weights(&self, index_id: U128) -> Result<Bytes, Vec<u8>> {
        let storage = Keep::storage();
        storage.check_version()?;

        let clerk_storage = ClerkStorage::storage();
        let vault = storage.vaults.get(index_id);

        let data = clerk_storage
            .fetch_bytes(vault.weights.get())
            .ok_or_else(|| b"Weights not set")?;

        Ok(data.into())
    }

    pub fn get_index_quote(&self, index_id: U128, vendor_id: U128) -> Result<Bytes, Vec<u8>> {
        let storage = Keep::storage();
        storage.check_version()?;

        let clerk_storage = ClerkStorage::storage();
        let vault = storage.vaults.get(index_id);

        let quote_id = vault.vendor_quotes.get(vendor_id);

        let data = clerk_storage
            .fetch_bytes(quote_id)
            .ok_or_else(|| b"Quote not set")?;

        Ok(data.into())
    }

    pub fn get_trader_order(&self, index_id: U128, trader: Address) -> Result<Bytes, Vec<u8>> {
        let storage = Keep::storage();
        storage.check_version()?;

        let clerk_storage = ClerkStorage::storage();
        let vault = storage.vaults.get(index_id);

        let bid_id = vault.traders_bids.get(trader);

        let bid = if !bid_id.is_zero() {
            clerk_storage
                .fetch_bytes(bid_id)
                .ok_or_else(|| b"Bid not set")?
        } else {
            amount_vec!(0, 0, 0).to_vec()
        };

        let ask_id = vault.traders_asks.get(trader);
        let ask = if !ask_id.is_zero() {
            clerk_storage
                .fetch_bytes(ask_id)
                .ok_or_else(|| b"Ask not set")?
        } else {
            amount_vec!(0, 0, 0).to_vec()
        };

        Ok(Order::encode_vec_pair(bid, ask).into())
    }

    pub fn get_trader_count(&self, index_id: U128) -> Result<U128, Vec<u8>> {
        let storage = Keep::storage();
        storage.check_version()?;

        let vault = storage.vaults.get(index_id);

        let result = U128::from(vault.traders.len());
        Ok(result)
    }

    pub fn get_trader_at(&self, index_id: U128, offset: u128) -> Result<Address, Vec<u8>> {
        let storage = Keep::storage();
        storage.check_version()?;

        let vault = storage.vaults.get(index_id);

        if let Some(address) = vault.traders.get(offset) {
            Ok(address)
        } else {
            Err(b"Trader has no orders".into())
        }
    }

    pub fn get_vendor_order(&self, index_id: U128, vendor_id: U128) -> Result<Bytes, Vec<u8>> {
        let storage = Keep::storage();
        storage.check_version()?;

        let clerk_storage = ClerkStorage::storage();
        let vault = storage.vaults.get(index_id);

        let bid_id = vault.vendors_bids.get(vendor_id);

        let bid = if !bid_id.is_zero() {
            clerk_storage
                .fetch_bytes(bid_id)
                .ok_or_else(|| b"Vendor bid not set")?
        } else {
            amount_vec!(0, 0, 0).to_vec()
        };

        let ask_id = vault.vendors_asks.get(vendor_id);
        let ask = if !ask_id.is_zero() {
            clerk_storage
                .fetch_bytes(ask_id)
                .ok_or_else(|| b"Vendor ask not set")?
        } else {
            amount_vec!(0, 0, 0).to_vec()
        };

        Ok(Order::encode_vec_pair(bid, ask).into())
    }

    pub fn get_vendor_count(&self, index_id: U128) -> Result<U128, Vec<u8>> {
        let storage = Keep::storage();
        storage.check_version()?;

        let vault = storage.vaults.get(index_id);

        let result = U128::from(vault.vendors.len());
        Ok(result)
    }

    pub fn get_vendor_at(&self, index_id: U128, offset: u128) -> Result<U128, Vec<u8>> {
        let storage = Keep::storage();
        storage.check_version()?;

        let vault = storage.vaults.get(index_id);

        if let Some(vendor_id) = vault.vendors.get(offset) {
            Ok(vendor_id)
        } else {
            Err(b"Vendor has no orders".into())
        }
    }

    pub fn get_total_order(&self, index_id: U128) -> Result<Bytes, Vec<u8>> {
        let storage = Keep::storage();
        storage.check_version()?;

        let clerk_storage = ClerkStorage::storage();
        let vault = storage.vaults.get(index_id);

        let bid_id = vault.total_bid.get();

        let bid = if !bid_id.is_zero() {
            clerk_storage
                .fetch_bytes(bid_id)
                .ok_or_else(|| b"Total bid not set")?
        } else {
            amount_vec!(0, 0, 0).to_vec()
        };

        let ask_id = vault.total_ask.get();
        let ask = if !ask_id.is_zero() {
            clerk_storage
                .fetch_bytes(ask_id)
                .ok_or_else(|| b"Total ask not set")?
        } else {
            amount_vec!(0, 0, 0).to_vec()
        };

        Ok(Order::encode_vec_pair(bid, ask).into())
    }

    //
    // Query methods (Banker)
    //

    pub fn get_vendor_assets(&mut self, vendor_id: U128) -> Result<Bytes, Vec<u8>> {
        let mut storage = Keep::storage();
        storage.check_version()?;

        let account = storage.accounts.setter(vendor_id);

        let clerk_storage = ClerkStorage::storage();

        let assets = clerk_storage
            .fetch_bytes(account.assets.get())
            .ok_or_else(|| b"No assets for vendor")?;

        Ok(assets.into())
    }

    pub fn get_vendor_margin(&mut self, vendor_id: U128) -> Result<Bytes, Vec<u8>> {
        let mut storage = Keep::storage();
        storage.check_version()?;

        let account = storage.accounts.setter(vendor_id);

        let clerk_storage = ClerkStorage::storage();
        let margin = clerk_storage
            .fetch_bytes(account.margin.get())
            .ok_or_else(|| b"No margin for vendor")?;

        Ok(margin.into())
    }

    pub fn get_vendor_supply(&mut self, vendor_id: U128) -> Result<Vec<Bytes>, Vec<u8>> {
        let mut storage = Keep::storage();
        storage.check_version()?;

        let account = storage.accounts.setter(vendor_id);

        let clerk_storage = ClerkStorage::storage();
        let supply_long = clerk_storage
            .fetch_bytes(account.supply_long.get())
            .ok_or_else(|| b"No supply long for vendor")?;

        let supply_short = clerk_storage
            .fetch_bytes(account.supply_short.get())
            .ok_or_else(|| b"No supply short for vendor")?;

        Ok(vec![supply_long.into(), supply_short.into()])
    }

    pub fn get_vendor_demand(&mut self, vendor_id: U128) -> Result<Vec<Bytes>, Vec<u8>> {
        let mut storage = Keep::storage();
        storage.check_version()?;

        let account = storage.accounts.setter(vendor_id);

        let clerk_storage = ClerkStorage::storage();
        let demand_long = clerk_storage
            .fetch_bytes(account.demand_long.get())
            .ok_or_else(|| b"No demand long for vendor")?;

        let demand_short = clerk_storage
            .fetch_bytes(account.demand_short.get())
            .ok_or_else(|| b"No demand short for vendor")?;

        Ok(vec![demand_long.into(), demand_short.into()])
    }

    pub fn get_vendor_delta(&mut self, vendor_id: U128) -> Result<Vec<Bytes>, Vec<u8>> {
        let mut storage = Keep::storage();

        let account = storage.accounts.setter(vendor_id);

        let clerk_storage = ClerkStorage::storage();
        let delta_long = clerk_storage
            .fetch_bytes(account.delta_long.get())
            .ok_or_else(|| b"No delta long for vendor")?;

        let delta_short = clerk_storage
            .fetch_bytes(account.delta_short.get())
            .ok_or_else(|| b"No delta short for vendor")?;

        Ok(vec![delta_long.into(), delta_short.into()])
    }

    //
    // Query methods (Clerk)
    //

    pub fn fetch_vector(&self, id: U128) -> Result<Bytes, Vec<u8>> {
        let storage = ClerkStorage::storage();

        let Some(vector) = storage.fetch_bytes(id) else {
            return Err(format!("Vector not found: {}", id))?;
        };

        Ok(Bytes::from(vector))
    }
}
