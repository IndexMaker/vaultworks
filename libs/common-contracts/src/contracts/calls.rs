use alloc::vec::Vec;

use alloy_primitives::Address;
use alloy_sol_types::SolCall;
use stylus_sdk::prelude::*;

pub trait InnerCall
where
    Self: HostAccess + TopLevelStorage,
{
    fn top_level_call<C>(&mut self, call: C) -> Result<Vec<u8>, Vec<u8>>
    where
        C: SolCall,
    {
        self.inner_call(self.vm().contract_address(), call)
    }

    fn inner_call<C>(&mut self, to: Address, call: C) -> Result<Vec<u8>, Vec<u8>>
    where
        C: SolCall;

    fn inner_call_ret<C>(&mut self, to: Address, call: C) -> Result<C::Return, Vec<u8>>
    where
        C: SolCall;

    fn external_call<C>(&mut self, to: Address, call: C) -> Result<Vec<u8>, Vec<u8>>
    where
        C: SolCall;

    fn external_call_ret<C>(&mut self, to: Address, call: C) -> Result<C::Return, Vec<u8>>
    where
        C: SolCall;

    fn static_call<C>(&self, to: Address, call: C) -> Result<Vec<u8>, Vec<u8>>
    where
        C: SolCall;

    fn static_call_ret<C>(&self, to: Address, call: C) -> Result<C::Return, Vec<u8>>
    where
        C: SolCall;
}

impl<T> InnerCall for T
where
    T: HostAccess + TopLevelStorage,
{
    fn inner_call<C>(&mut self, to: Address, call: C) -> Result<Vec<u8>, Vec<u8>>
    where
        C: SolCall,
    {
        let data = call.abi_encode();
        let result = unsafe { self.vm().delegate_call(&self, to, &data) }?;
        Ok(result)
    }
    
    fn inner_call_ret<C>(&mut self, to: Address, call: C) -> Result<C::Return, Vec<u8>>
    where
        C: SolCall,
    {
        let result_bytes = self.inner_call(to, call)?;
        let result = C::abi_decode_returns(&result_bytes, true)
                .map_err(|_| b"Failed to decode return data")?;
        Ok(result)
    }

    fn external_call<C>(&mut self, to: Address, call: C) -> Result<Vec<u8>, Vec<u8>>
    where
        C: SolCall,
    {
        let data = call.abi_encode();
        let result = self.vm().call(&self, to, &data)?;
        Ok(result)
    }
    
    fn external_call_ret<C>(&mut self, to: Address, call: C) -> Result<C::Return, Vec<u8>>
    where
        C: SolCall,
    {
        let result_bytes = self.external_call(to, call)?;
        let result = C::abi_decode_returns(&result_bytes, true)
                .map_err(|_| b"Failed to decode return data")?;
        Ok(result)
    }

    fn static_call<C>(&self, to: Address, call: C) -> Result<Vec<u8>, Vec<u8>>
    where
        C: SolCall,
    {
        let data = call.abi_encode();
        let result = self.vm().static_call(&self, to, &data)?;
        Ok(result)
    }

    fn static_call_ret<C>(&self, to: Address, call: C) -> Result<C::Return, Vec<u8>>
    where
        C: SolCall,
    {
        let result_bytes = self.static_call(to, call)?;
        let result = C::abi_decode_returns(&result_bytes, true)
                .map_err(|_| b"Failed to decode return data")?;
        Ok(result)
    }
}
