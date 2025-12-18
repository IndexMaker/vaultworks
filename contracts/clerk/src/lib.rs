// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U128};
use deli::{contracts::granary::GranaryStorage, labels::Labels, vector::Vector};
use stylus_sdk::prelude::*;

use crate::program::{ErrorCode, Program, VectorIO};

pub mod program;

#[cfg(test)]
pub mod test;

impl VectorIO for GranaryStorage {
    fn load_labels(&self, id: u128) -> Result<Labels, ErrorCode> {
        let key = U128::from(id);
        let Some(vector) = self.fetch_bytes(key) else {
            return Err(ErrorCode::NotFound);
        };
        Ok(Labels::from_vec(vector))
    }

    fn load_vector(&self, id: u128) -> Result<Vector, ErrorCode> {
        let key = U128::from(id);
        let Some(vector) = self.fetch_bytes(key) else {
            return Err(ErrorCode::NotFound);
        };
        Ok(Vector::from_vec(vector))
    }

    fn load_code(&self, id: u128) -> Result<Vec<u8>, ErrorCode> {
        let key = U128::from(id);
        let Some(vector) = self.fetch_bytes(key) else {
            return Err(ErrorCode::NotFound);
        };
        Ok(vector)
    }

    fn store_labels(&mut self, id: u128, input: Labels) -> Result<(), ErrorCode> {
        let key = U128::from(id);
        self.store_bytes(key, input.to_vec());
        Ok(())
    }

    fn store_vector(&mut self, id: u128, input: Vector) -> Result<(), ErrorCode> {
        let key = U128::from(id);
        self.store_bytes(key, input.to_vec());
        Ok(())
    }
}

#[storage]
#[entrypoint]
pub struct Clerk;

impl Clerk {
    fn _attendee(&self) -> Address {
        self.vm().msg_sender()
    }
}

#[public]
impl Clerk {
    pub fn execute(&mut self, code: Vec<u8>, num_registry: u128) -> Result<(), Vec<u8>> {
        let mut storage = GranaryStorage::storage();
        storage.only_owner(self._attendee())?;

        let mut program = Program::new(&mut storage);
        program
            .execute(code, num_registry as usize)
            .map_err(|err| format!("Program error: {:?}", err))?;

        Ok(())
    }
}
