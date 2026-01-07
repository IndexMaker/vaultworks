use alloc::vec::Vec;

use alloy_primitives::{Address, Bytes};

use crate::{
    contracts::calls::InnerCall,
    interfaces::{abacus::IAbacus, scribe::IScribe, worksman::IWorksman},
};

pub trait KeepCalls {
    fn attendee(&self) -> Address;

    fn execute_vector_program(
        &mut self,
        clerk: Address,
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

    fn execute_vector_program(
        &mut self,
        clerk: Address,
        code: impl Into<Bytes>,
        num_registry: u128,
    ) -> Result<(), Vec<u8>> {
        let call = IAbacus::executeCall {
            code: code.into(),
            num_registry,
        };
        self.external_call(clerk, call)?;
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
