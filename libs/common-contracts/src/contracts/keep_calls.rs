use alloc::vec::Vec;

use alloy_primitives::{Address, Bytes};

use common::vector::Vector;

use crate::{
    contracts::calls::InnerCall,
    interfaces::{abacus::IAbacus, clerk::IClerk, scribe::IScribe, worksman::IWorksman},
};

pub trait KeepCalls {
    fn attendee(&self) -> Address;

    fn submit_vector_bytes(
        &mut self,
        gate_to_clerk_chamber: Address,
        vector_id: u128,
        data: impl Into<Bytes>,
    ) -> Result<(), Vec<u8>>;

    fn fetch_vector_bytes(
        &self,
        gate_to_clerk_chamber: Address,
        vector_id: u128,
    ) -> Result<Vec<u8>, Vec<u8>>;

    fn fetch_vector_from_clerk(
        &self,
        gate_to_clerk_chamber: Address,
        vector_id: u128,
    ) -> Result<Vector, Vec<u8>> {
        let data = self.fetch_vector_bytes(gate_to_clerk_chamber, vector_id)?;
        Ok(Vector::from_vec(data))
    }

    fn execute_vector_program(
        &mut self,
        gate_to_clerk_chamber: Address,
        code: impl Into<Bytes>,
        num_registry: u128,
    ) -> Result<(), Vec<u8>>;

    fn build_vault(
        &mut self,
        worksman: Address,
        index_id: u128,
        info: impl Into<Bytes>,
    ) -> Result<Address, Vec<u8>>;

    fn verify_signature(
        &mut self,
        scribe: Address,
        data: impl Into<Bytes>,
    ) -> Result<bool, Vec<u8>>;
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
        gate_to_clerk_chamber: Address,
        vector_id: u128,
        data: impl Into<Bytes>,
    ) -> Result<(), Vec<u8>> {
        let call = IClerk::storeCall {
            id: vector_id,
            data: data.into(),
        };
        self.external_call(gate_to_clerk_chamber, call)?;
        Ok(())
    }

    fn fetch_vector_bytes(
        &self,
        gate_to_clerk_chamber: Address,
        vector_id: u128,
    ) -> Result<Vec<u8>, Vec<u8>> {
        let call = IClerk::loadCall { id: vector_id };
        let IClerk::loadReturn { _0: result } =
            self.static_call_ret(gate_to_clerk_chamber, call)?;
        Ok(result.to_vec())
    }

    fn execute_vector_program(
        &mut self,
        gate_to_clerk_chamber: Address,
        code: impl Into<Bytes>,
        num_registry: u128,
    ) -> Result<(), Vec<u8>> {
        let call = IAbacus::executeCall {
            code: code.into(),
            num_registry,
        };
        self.external_call(gate_to_clerk_chamber, call)?;
        Ok(())
    }

    fn build_vault(
        &mut self,
        worksman: Address,
        index_id: u128,
        info: impl Into<Bytes>,
    ) -> Result<Address, Vec<u8>> {
        let IWorksman::buildVaultReturn { _0: result } = self.inner_call_ret(
            worksman,
            IWorksman::buildVaultCall {
                index: index_id,
                info: info.into(),
            },
        )?;
        Ok(result)
    }

    fn verify_signature(
        &mut self,
        scribe: Address,
        data: impl Into<Bytes>,
    ) -> Result<bool, Vec<u8>> {
        let IScribe::verifySignatureReturn {
            _0: verfication_result,
        } = self.inner_call_ret(scribe, IScribe::verifySignatureCall { data: data.into() })?;
        Ok(verfication_result)
    }
}
