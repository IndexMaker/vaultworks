// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U128};
use common::{abacus::program_error::ErrorCode, labels::Labels, vector::Vector};
use common_contracts::contracts::clerk::ClerkStorage;
use stylus_sdk::{abi::Bytes, prelude::*};

use abacus_runtime::runtime::{VectorIO, VectorVM};

struct ClerkStorageRef<'a>(&'a mut ClerkStorage);

impl<'a> VectorIO for ClerkStorageRef<'a> {
    fn load_labels(&self, id: u128) -> Result<Labels, ErrorCode> {
        let key = U128::from(id);
        let Some(vector) = self.0.fetch_bytes(key) else {
            return Err(ErrorCode::NotFound);
        };
        Ok(Labels::from_vec(vector))
    }

    fn load_vector(&self, id: u128) -> Result<Vector, ErrorCode> {
        let key = U128::from(id);
        let Some(vector) = self.0.fetch_bytes(key) else {
            return Err(ErrorCode::NotFound);
        };
        Ok(Vector::from_vec(vector))
    }

    fn load_code(&self, id: u128) -> Result<Vec<u8>, ErrorCode> {
        let key = U128::from(id);
        let Some(vector) = self.0.fetch_bytes(key) else {
            return Err(ErrorCode::NotFound);
        };
        Ok(vector)
    }

    fn store_labels(&mut self, id: u128, input: Labels) -> Result<(), ErrorCode> {
        let key = U128::from(id);
        self.0.store_bytes(key, input.to_vec());
        Ok(())
    }

    fn store_vector(&mut self, id: u128, input: Vector) -> Result<(), ErrorCode> {
        let key = U128::from(id);
        self.0.store_bytes(key, input.to_vec());
        Ok(())
    }
}

#[storage]
#[entrypoint]
pub struct Abacus;

impl Abacus {
    fn _attendee(&self) -> Address {
        self.vm().msg_sender()
    }
}

#[public]
impl Abacus {
    pub fn execute(&mut self, code: Bytes, num_registry: u128) -> Result<(), Vec<u8>> {
        let mut storage = ClerkStorage::storage();
        storage.only_owner(self._attendee())?;

        let mut ref_storage = ClerkStorageRef(&mut storage);
        let mut program = VectorVM::new(&mut ref_storage);
        program
            .execute(code.to_vec(), num_registry as usize)
            .map_err(|err| format!("Program error: {:?}", err))?;

        Ok(())
    }
}
