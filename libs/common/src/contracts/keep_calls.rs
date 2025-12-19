use alloc::vec::Vec;

use alloy_primitives::Address;

use crate::{
    contracts::calls::InnerCall,
    interfaces::{abacus::IAbacus, clerk::IClerk, scribe::IScribe, worksman::IWorksman},
    vector::Vector,
};

pub trait KeepCalls {
    fn attendee(&self) -> Address;

    fn submit_vector_bytes(
        &mut self,
        gate_to_clerk: Address,
        vector_id: u128,
        data: Vec<u8>,
    ) -> Result<Vec<u8>, Vec<u8>>;

    fn fetch_vector_bytes(
        &self,
        gate_to_clerk: Address,
        vector_id: u128,
    ) -> Result<Vec<u8>, Vec<u8>>;

    fn fetch_vector_from_clerk(
        &self,
        gate_to_clerk: Address,
        vector_id: u128,
    ) -> Result<Vector, Vec<u8>> {
        let data = self.fetch_vector_bytes(gate_to_clerk, vector_id)?;
        Ok(Vector::from_vec(data))
    }

    fn execute_vector_program(
        &mut self,
        gate_to_clerk: Address,
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
        gate_to_clerk: Address,
        vector_id: u128,
        data: Vec<u8>,
    ) -> Result<Vec<u8>, Vec<u8>> {
        let result = self.external_call(
            gate_to_clerk,
            IClerk::storeCall {
                id: vector_id,
                data,
            },
        )?;
        Ok(result)
    }

    fn fetch_vector_bytes(
        &self,
        gate_to_clerk: Address,
        vector_id: u128,
    ) -> Result<Vec<u8>, Vec<u8>> {
        let result = self.static_call(gate_to_clerk, IClerk::loadCall { id: vector_id })?;
        Ok(result)
    }

    fn execute_vector_program(
        &mut self,
        gate_to_clerk: Address,
        code: Vec<u8>,
        num_registry: u128,
    ) -> Result<(), Vec<u8>> {
        self.external_call(gate_to_clerk, IAbacus::executeCall { code, num_registry })?;
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
        let address_bytes: [u8; 20] = gate_to_vault_bytes[12..32]
            .try_into()
            .map_err(|_| b"Bad gate to vault address")?;
        let result = Address::from(address_bytes);
        Ok(result)
    }

    fn verify_signature(&mut self, scribe: Address, data: Vec<u8>) -> Result<bool, Vec<u8>> {
        let verfication_result_bytes = self.inner_call(scribe, IScribe::verifySignatureCall { data })?;
        let result_byte: [u8; 1] = verfication_result_bytes [31..32]
            .try_into()
            .map_err(|_| b"Bad signature verification")?;
        Ok(result_byte[0] == 1u8)
    }
}
