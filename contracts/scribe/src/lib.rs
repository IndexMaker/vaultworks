// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use common_contracts::contracts::keep::Keep;
use stylus_sdk::{abi::Bytes, prelude::*};

use common_bls::{
    affine::{public_key_from_data, signature_from_data},
    bls::verify_signature,
};

#[storage]
#[entrypoint]
pub struct Scribe;

#[public]
impl Scribe {
    pub fn verify_signature(
        &mut self,
        public_key: Bytes,
        signature: Bytes,
    ) -> Result<bool, Vec<u8>> {
        let keep = Keep::storage();
        if keep.scribe.get().is_zero() {
            Err(b"Scribe not appointed")?;
        }

        let public_key = public_key_from_data(&public_key)?;
        let signature = signature_from_data(&signature)?;

        let res = verify_signature(b"message", &public_key, &signature);
        Ok(res)
    }
}
