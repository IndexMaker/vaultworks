// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::Address;
use openzeppelin_stylus::proxy::{IProxy, erc1967::{self, Erc1967Proxy}};
use stylus_sdk::{
    ArbResult, abi::Bytes, prelude::*
};

#[entrypoint]
#[storage]
struct Gate {
    erc1967: Erc1967Proxy,
}

/// One to one delegating proxy
/// 
/// All calls are delegated to implementation
/// 
#[public]
impl Gate {
    #[constructor]
    pub fn constructor(
        &mut self,
        implementation: Address,
        data: Bytes,
    ) -> Result<(), erc1967::utils::Error> {
        self.erc1967.constructor(implementation, &data)
    }
    
    /// If we cannot use constructor() due to Cargo/Stylus limitation
    /// we shall use initialize() once-and-only-once (this is tested
    /// by checking implementation is zero).
    pub fn initialize(
        &mut self,
        implementation: Address,
        data: Bytes,
    ) -> Result<(), Vec<u8>> {
        if !self.implementation()?.is_zero() {
            Err(b"Can only call initialize once")?;
        }
        self.erc1967.constructor(implementation, &data)?;
        Ok(())
    }

    fn implementation(&self) -> Result<Address, Vec<u8>> {
        self.erc1967.implementation()
    }

    #[payable]
    #[fallback]
    fn fallback(&mut self, calldata: &[u8]) -> ArbResult {
        unsafe { self.erc1967.do_fallback(calldata) }
    }
}

#[cfg(test)]
mod test {}
