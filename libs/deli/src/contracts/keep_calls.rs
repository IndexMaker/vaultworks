use alloc::vec::Vec;

use alloy_primitives::{Address, U8};

use crate::{
    contracts::{
        calls::InnerCall,
        interfaces::{
            clerk::IClerk,
            granary::IGranary,
            scribe::IScribe,
            worksman::IWorksman,
        },
    },
    vector::Vector,
};

pub trait KeepCalls {
    fn attendee(&self) -> Address;

    fn submit_vector_bytes(
        &mut self,
        gate_to_granary: Address,
        vector_id: u128,
        data: Vec<u8>,
    ) -> Result<Vec<u8>, Vec<u8>>;

    fn fetch_vector_bytes(
        &self,
        gate_to_granary: Address,
        vector_id: u128,
    ) -> Result<Vec<u8>, Vec<u8>>;

    fn fetch_vector_from_granary(
        &self,
        gate_to_granary: Address,
        vector_id: u128,
    ) -> Result<Vector, Vec<u8>> {
        let data = self.fetch_vector_bytes(gate_to_granary, vector_id)?;
        Ok(Vector::from_vec(data))
    }

    fn execute_vector_program(
        &mut self,
        gate_to_granary: Address,
        code: Vec<u8>,
        num_registry: u128,
    ) -> Result<(), Vec<u8>>;

    fn build_vault(
        &mut self,
        worksman: Address,
        index_id: u128,
        info: Vec<u8>,
    ) -> Result<Address, Vec<u8>>;

    fn verify_signature(&mut self, scribe: Address, data: Vec<u8>) -> Result<bool, Vec<u8>>;
}

impl<T> KeepCalls for T
where
    T: InnerCall,
{
    fn attendee(&self) -> Address {
        self.vm().msg_sender()
    }

    fn submit_vector_bytes(
        &mut self,
        gate_to_granary: Address,
        vector_id: u128,
        data: Vec<u8>,
    ) -> Result<Vec<u8>, Vec<u8>> {
        let result = self.external_call(
            gate_to_granary,
            IGranary::storeCall {
                id: vector_id,
                data,
            },
        )?;
        Ok(result)
    }

    fn fetch_vector_bytes(
        &self,
        gate_to_granary: Address,
        vector_id: u128,
    ) -> Result<Vec<u8>, Vec<u8>> {
        let result = self.static_call(gate_to_granary, IGranary::fetchCall { id: vector_id })?;
        Ok(result)
    }

    fn execute_vector_program(
        &mut self,
        gate_to_granary: Address,
        code: Vec<u8>,
        num_registry: u128,
    ) -> Result<(), Vec<u8>> {
        self.external_call(gate_to_granary, IClerk::executeCall { code, num_registry })?;
        Ok(())
    }

    fn build_vault(
        &mut self,
        worksman: Address,
        index_id: u128,
        info: Vec<u8>,
    ) -> Result<Address, Vec<u8>> {
        let gate_to_vault_bytes = self.inner_call(
            worksman,
            IWorksman::buildVaultCall {
                index: index_id,
                info,
            },
        )?;
        let result = Address::from_slice(&gate_to_vault_bytes);
        Ok(result)
    }

    fn verify_signature(&mut self, scribe: Address, data: Vec<u8>) -> Result<bool, Vec<u8>> {
        let result_bytes = self.inner_call(scribe, IScribe::verifySignatureCall { data })?;
        let verfication_result = U8::from_be_slice(&result_bytes);
        Ok(verfication_result == U8::ONE)
    }
}
