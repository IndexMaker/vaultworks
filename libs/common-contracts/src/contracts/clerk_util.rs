use alloc::{vec, vec::Vec};
use common::{amount::Amount, labels::Labels, vector::Vector};

use super::{clerk::ClerkStorage, keep::Vault};
use alloy_primitives::{Address, U128};

pub fn new_vector_bytes(clerk_storage: &mut ClerkStorage, data: impl AsRef<[u8]>) -> U128 {
    let vector_id = clerk_storage.next_vector();

    clerk_storage.store_bytes(vector_id.to(), data);

    vector_id
}

pub fn new_labels(clerk_storage: &mut ClerkStorage, data: Labels) -> U128 {
    let vector_id = clerk_storage.next_vector();

    clerk_storage.store_bytes(vector_id.to(), data.to_vec());

    vector_id
}

#[inline]
pub fn new_labels_empty(clerk_storage: &mut ClerkStorage) -> U128 {
    new_labels(clerk_storage, Labels::new())
}

pub fn new_vector(clerk_storage: &mut ClerkStorage, data: Vector) -> U128 {
    let vector_id = clerk_storage.next_vector();

    clerk_storage.store_vector(vector_id.to(), data);

    vector_id
}

#[inline]
pub fn new_vector_empty(clerk_storage: &mut ClerkStorage) -> U128 {
    new_vector(clerk_storage, Vector::new())
}

#[inline]
pub fn new_vector_3z(clerk_storage: &mut ClerkStorage) -> U128 {
    new_vector(
        clerk_storage,
        Vector {
            data: vec![Amount::ZERO, Amount::ZERO, Amount::ZERO],
        },
    )
}

pub fn lazy_init_trader_bid(
    vault: &mut Vault,
    clerk_storage: &mut ClerkStorage,
    trader_address: Address,
) -> U128 {
    let mut set_bid_id = vault.traders_bids.setter(trader_address);

    let bid_id = set_bid_id.get();
    if !bid_id.is_zero() {
        return bid_id;
    }

    let bid_id = new_vector_3z(clerk_storage);
    set_bid_id.set(bid_id);

    if vault.traders_asks.get(trader_address).is_zero() {
        vault.traders.push(trader_address);
    }

    bid_id
}

pub fn lazy_init_trader_ask(
    vault: &mut Vault,
    clerk_storage: &mut ClerkStorage,
    trader_address: Address,
) -> U128 {
    let mut set_ask_id = vault.traders_asks.setter(trader_address);

    let ask_id = set_ask_id.get();
    if !ask_id.is_zero() {
        return ask_id;
    }

    let ask_id = new_vector_3z(clerk_storage);
    set_ask_id.set(ask_id);

    if vault.traders_bids.get(trader_address).is_zero() {
        vault.traders.push(trader_address);
    }

    ask_id
}

pub fn lazy_init_vendor_bid(
    vault: &mut Vault,
    clerk_storage: &mut ClerkStorage,
    vendor_id: U128,
) -> U128 {
    let mut set_bid_id = vault.vendors_bids.setter(vendor_id);

    let bid_id = set_bid_id.get();
    if !bid_id.is_zero() {
        return bid_id;
    }

    let bid_id = new_vector_3z(clerk_storage);
    set_bid_id.set(bid_id);

    if vault.vendor_quotes.get(vendor_id).is_zero() && vault.vendors_asks.get(vendor_id).is_zero() {
        vault.vendors.push(vendor_id);
    }

    bid_id
}

pub fn get_vendor_quote_id(vault: &mut Vault, vendor_id: U128) -> Result<U128, Vec<u8>> {
    let quote_id = vault.vendor_quotes.get(vendor_id);
    if quote_id.is_zero() {
        Err(b"No index quote for vendor")?;
    }

    return Ok(quote_id);
}

pub fn lazy_init_vendor_ask(
    vault: &mut Vault,
    clerk_storage: &mut ClerkStorage,
    vendor_id: U128,
) -> U128 {
    let mut set_ask_id = vault.vendors_asks.setter(vendor_id);

    let ask_id = set_ask_id.get();
    if !ask_id.is_zero() {
        return ask_id;
    }

    let ask_id = new_vector_3z(clerk_storage);
    set_ask_id.set(ask_id);

    if vault.vendor_quotes.get(vendor_id).is_zero() && vault.vendors_bids.get(vendor_id).is_zero() {
        vault.vendors.push(vendor_id);
    }

    ask_id
}

pub fn lazy_init_vendor_quote(
    vault: &mut Vault,
    clerk_storage: &mut ClerkStorage,
    vendor_id: U128,
) -> U128 {
    let mut set_quote_id = vault.vendor_quotes.setter(vendor_id);

    let quote_id = set_quote_id.get();
    if !quote_id.is_zero() {
        return quote_id;
    }

    let quote_id = new_vector_3z(clerk_storage);
    set_quote_id.set(quote_id);

    if vault.vendors_bids.get(vendor_id).is_zero() && vault.vendors_asks.get(vendor_id).is_zero() {
        vault.vendors.push(vendor_id);
    }

    quote_id
}
