// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::{string::String, vec::Vec};

use alloy_primitives::{uint, Address, B256, U256, U32, U8};
use alloy_sol_types::{sol, SolCall, SolEvent};
use common::{amount::Amount, vector::Vector};
use common_contracts::{
    contracts::{calls::InnerCall, keep_calls::KeepCalls, storage::StorageSlot},
    interfaces::factor::IFactor,
};
use stylus_sdk::{
    abi::Bytes,
    keccak_const,
    prelude::*,
    storage::{
        StorageAddress, StorageBool, StorageMap, StorageString, StorageU128, StorageU256,
        StorageU32,
    },
};

pub const VAULT_STORAGE_SLOT: U256 = {
    const HASH: [u8; 32] = keccak_const::Keccak256::new()
        .update(b"Vault.STORAGE_SLOT")
        .finalize();
    U256::from_be_bytes(HASH).wrapping_sub(uint!(1_U256))
};

pub const VERSION_NUMBER: U32 = uint!(1_U32);
pub const UPGRADE_INTERFACE_VERSION: &str = "5.0.0";

const ORDER_REMAIN_OFFSET: usize = 0;
const ORDER_SPENT_OFFSET: usize = 1;
const ORDER_REALIZED_OFFSET: usize = 2;

const QUOTE_CAPACITY_OFFSET: usize = 0;
const QUOTE_PRICE_OFFSET: usize = 1;
const QUOTE_SLOPE_OFFSET: usize = 2;

sol! {
    interface IERC20 {
        event Transfer(address indexed from, address indexed to, uint256 value);

        event Approval(address indexed owner, address indexed spender, uint256 value);

        function transferFrom(address from, address to, uint256 value) external returns (bool);
    }

    event DepositRequest(address controller, address owner, uint256 requestId, address sender, uint256 assets);
}

#[storage]
struct Request {
    pending_request: StorageU256,
    claimable_request: StorageU256,
}

impl Request {
    fn pending(&self) -> U256 {
        self.pending_request.get()
    }

    fn claimable(&self) -> U256 {
        self.claimable_request.get()
    }

    fn request(&mut self, assets: U256) -> Result<(), Vec<u8>> {
        let current = self.pending_request.get();
        let result = current.checked_add(assets).ok_or_else(|| b"MathOverflow")?;
        self.pending_request.set(result);
        Ok(())
    }

    fn claim(&mut self, assets: U256) -> Result<(), Vec<u8>> {
        let current = self.claimable_request.get();
        let result = current
            .checked_sub(assets)
            .ok_or_else(|| b"Insufficient Claimable")?;
        self.claimable_request.set(result);
        Ok(())
    }
}

#[storage]
struct Operator {
    operators: StorageMap<Address, StorageBool>,
}

impl Operator {
    fn is_operator(&self, operator: Address) -> bool {
        self.operators.get(operator)
    }

    fn set_operator(&mut self, operator: Address, approved: bool) {
        let mut setter = self.operators.setter(operator);
        setter.set(approved);
    }
}

#[storage]
struct VaultStorage {
    index_id: StorageU128,
    vendor_id: StorageU128,
    version: StorageU32,
    name: StorageString,
    symbol: StorageString,
    owner: StorageAddress,
    collateral_asset: StorageAddress,
    controller: StorageMap<Address, Operator>,
    deposit_request: StorageMap<Address, Request>,
    redeem_request: StorageMap<Address, Request>,
    gate_to_castle: StorageAddress,
}

#[storage]
#[entrypoint]
pub struct Vault;

impl Vault {
    fn _storage() -> VaultStorage {
        StorageSlot::get_slot::<VaultStorage>(VAULT_STORAGE_SLOT)
    }

    fn _next_request_id(&mut self) -> U256 {
        U256::ZERO
    }

    fn _only_owner(&self, vault: &VaultStorage) -> Result<(), Vec<u8>> {
        let owner = vault.owner.get();
        if !owner.is_zero() && owner != self.attendee() {
            Err(b"Only owner")?;
        }
        Ok(())
    }

    fn _set_version(&mut self, vault: &mut VaultStorage) -> Result<(), Vec<u8>> {
        if vault.version.get() > VERSION_NUMBER {
            Err(b"Version cannot be downgraded")?;
        }
        vault.version.set(VERSION_NUMBER);
        Ok(())
    }

    fn _set_collateral_asset(
        &mut self,
        vault: &mut VaultStorage,
        asset: Address,
    ) -> Result<(), Vec<u8>> {
        vault.collateral_asset.set(asset);
        Ok(())
    }

    fn _transfer_ownership(
        &mut self,
        vault: &mut VaultStorage,
        new_owner: Address,
    ) -> Result<(), Vec<u8>> {
        vault.owner.set(new_owner);
        Ok(())
    }

    fn _renounce_ownership(&mut self, vault: &mut VaultStorage) -> Result<(), Vec<u8>> {
        vault.owner.set(Address::ZERO);
        Ok(())
    }

    fn _get_order(
        &self,
        vault: &VaultStorage,
        account: Address,
    ) -> Result<(Vector, Vector), Vec<u8>> {
        let ret = self.static_call_ret(
            vault.gate_to_castle.get(),
            IFactor::getTraderOrderCall {
                index_id: vault.index_id.get().to(),
                trader: account,
            },
        )?;

        let bid = Vector::from_vec(ret._0);
        let ask = Vector::from_vec(ret._1);

        Ok((bid, ask))
    }

    fn _get_total_order(&self, vault: &VaultStorage) -> Result<(Vector, Vector), Vec<u8>> {
        let ret = self.static_call_ret(
            vault.gate_to_castle.get(),
            IFactor::getTotalOrderCall {
                index_id: vault.index_id.get().to(),
            },
        )?;

        let bid = Vector::from_vec(ret._0);
        let ask = Vector::from_vec(ret._1);

        Ok((bid, ask))
    }

    fn _get_quote(&self, vault: &VaultStorage) -> Result<Vector, Vec<u8>> {
        let ret = self.static_call(
            vault.gate_to_castle.get(),
            IFactor::getIndexQuoteCall {
                index_id: vault.index_id.get().to(),
                vendor_id: vault.vendor_id.get().to(),
            },
        )?;

        let quote = Vector::from_vec(ret);

        Ok(quote)
    }
}

#[public]
impl Vault {
    #[constructor]
    fn constructor(&mut self) {}

    // ERC4626

    fn asset(&self) -> Address {
        let vault = Self::_storage();
        vault.collateral_asset.get()
    }

    pub fn assets(&self, account: Address) -> Result<U256, Vec<u8>> {
        let vault = Self::_storage();

        let (bid, ask) = self._get_order(&vault, account)?;
        let quote = self._get_quote(&vault)?;

        let itp_unburnt = bid.data[ORDER_REALIZED_OFFSET]
            .checked_sub(ask.data[ORDER_SPENT_OFFSET])
            .ok_or_else(|| b"MathUnderflow")?;

        let assets_base_value = quote.data[QUOTE_PRICE_OFFSET]
            .checked_mul(itp_unburnt)
            .ok_or_else(|| b"MathOverflow")?;

        Ok(assets_base_value.to_u256())
    }

    pub fn total_assets(&self) -> Result<U256, Vec<u8>> {
        let vault = Self::_storage();

        let (bid, ask) = self._get_total_order(&vault)?;
        let quote = self._get_quote(&vault)?;

        let itp_unburnt = bid.data[ORDER_REALIZED_OFFSET]
            .checked_sub(ask.data[ORDER_SPENT_OFFSET])
            .ok_or_else(|| b"MathUnderflow")?;

        let assets_base_value = quote.data[QUOTE_PRICE_OFFSET]
            .checked_mul(itp_unburnt)
            .ok_or_else(|| b"MathOverflow")?;

        Ok(assets_base_value.to_u256())
    }

    pub fn convert_to_assets(&self, shares: U256) -> Result<U256, Vec<u8>> {
        let vault = Self::_storage();

        let quote = self._get_quote(&vault)?;
        let amount = Amount::try_from_u256(shares).ok_or_else(|| b"MathOverflow")?;

        let assets_base_value = quote.data[QUOTE_PRICE_OFFSET]
            .checked_mul(amount)
            .ok_or_else(|| b"MathOverflow")?;

        Ok(assets_base_value.to_u256())
    }

    pub fn convert_to_shares(&self, assets: U256) -> Result<U256, Vec<u8>> {
        let vault = Self::_storage();

        let quote = self._get_quote(&vault)?;
        let amount = Amount::try_from_u256(assets).ok_or_else(|| b"MathOverflow")?;

        let assets_base_value = amount
            .checked_div(quote.data[QUOTE_PRICE_OFFSET])
            .ok_or_else(|| b"MathOverflow")?;

        Ok(assets_base_value.to_u256())
    }

    fn request_deposit(
        &mut self,
        assets: U256,
        controller: Address,
        owner: Address,
    ) -> Result<U256, Vec<u8>> {
        let mut vault = Self::_storage();

        let request_id = self._next_request_id();

        let mut request = vault.deposit_request.setter(owner);
        request.request(assets)?;

        let request_event = DepositRequest {
            controller,
            owner,
            requestId: request_id,
            sender: self.attendee(),
            assets,
        };

        self.vm().emit_log(&request_event.encode_data(), 1);

        Ok(request_id)
    }

    fn max_deposit(&self, receiver: Address) -> U256 {
        U256::ZERO
    }

    fn preview_deposit(&self, assets: U256) -> Result<U256, Vec<u8>> {
        Err(b"Must deposit via Keeper service".into())
    }

    fn deposit(&mut self, assets: U256, receiver: Address) -> Result<U256, Vec<u8>> {
        Err(b"Must deposit via Keeper service".into())
    }

    fn max_redeem(&self, owner: Address) -> U256 {
        U256::ZERO
    }

    fn preview_redeem(&self, shares: U256) -> Result<U256, Vec<u8>> {
        Err(b"Must redeem via Keeper service".into())
    }

    fn redeem(&mut self, shares: U256, receiver: Address, owner: Address) -> Result<U256, Vec<u8>> {
        Err(b"Must redeem via Keeper service".into())
    }

    // These don't need to be implemented
    // @{
    // fn max_mint(&self, receiver: Address) -> Result<U256, Vec<u8>> {
    //     Err(b"Mint Unsupported".into())
    // }

    // fn preview_mint(&self, shares: U256) -> Result<U256, Vec<u8>> {
    //     Err(b"Mint Unsupported".into())
    // }

    // fn mint(&mut self, shares: U256, receiver: Address) -> Result<U256, Vec<u8>> {
    //     Err(b"Mint Unsupported".into())
    // }

    // fn max_withdraw(&self, owner: Address) -> Result<U256, Vec<u8>> {
    //     Err(b"Withdraw Unsupported".into())
    // }

    // fn preview_withdraw(&self, assets: U256) -> Result<U256, Vec<u8>> {
    //     Err(b"Withdraw Unsupported".into())
    // }

    // fn withdraw(
    //     &mut self,
    //     assets: U256,
    //     receiver: Address,
    //     owner: Address,
    // ) -> Result<U256, Vec<u8>> {
    //     Err(b"Withdraw Unsupported".into())
    // }
    // @}

    // ERC20

    pub fn name(&self) -> alloc::string::String {
        let vault = Self::_storage();
        vault.name.get_string()
    }

    pub fn symbol(&self) -> alloc::string::String {
        let vault = Self::_storage();
        vault.symbol.get_string()
    }

    pub fn decimals(&self) -> U8 {
        U8::from(18)
    }

    pub fn total_supply(&self) -> Result<U256, Vec<u8>> {
        let vault = Self::_storage();

        let ret = self.static_call_ret(
            vault.gate_to_castle.get(),
            IFactor::getTotalOrderCall {
                index_id: vault.index_id.get().to(),
            },
        )?;

        let bid = Vector::from_vec(ret._0);
        let ask = Vector::from_vec(ret._1);

        let itp_available = bid.data[ORDER_REALIZED_OFFSET]
            .checked_sub(ask.data[ORDER_REMAIN_OFFSET])
            .ok_or_else(|| b"MathUnderflow")?;

        Ok(itp_available.to_u256())
    }

    pub fn balance_of(&self, account: Address) -> Result<U256, Vec<u8>> {
        let vault = Self::_storage();

        let ret = self.static_call_ret(
            vault.gate_to_castle.get(),
            IFactor::getTraderOrderCall {
                index_id: vault.index_id.get().to(),
                trader: account,
            },
        )?;

        let bid = Vector::from_vec(ret._0);
        let ask = Vector::from_vec(ret._1);

        let itp_available = bid.data[ORDER_REALIZED_OFFSET]
            .checked_sub(ask.data[ORDER_REMAIN_OFFSET])
            .ok_or_else(|| b"MathUnderflow")?;

        Ok(itp_available.to_u256())
    }

    pub fn transfer(&mut self, to: Address, value: U256) -> Result<(), Vec<u8>> {
        let vault = Self::_storage();

        self.external_call(
            vault.gate_to_castle.get(),
            IFactor::submitTransferCall {
                index_id: vault.index_id.get().to(),
                receiver: to,
                amount: Amount::try_from_u256(value)
                    .ok_or_else(|| b"MathOverflow")?
                    .to_u128_raw(),
            },
        )?;

        let event = IERC20::Transfer {
            from: self.attendee(),
            to,
            value,
        };

        self.vm().emit_log(&event.encode_data(), 1);

        Ok(())
    }

    pub fn allowance(&self, owner: Address, spender: Address) -> Result<U256, Vec<u8>> {
        let vault = Self::_storage();
        let result = self.static_call_ret(
            vault.gate_to_castle.get(),
            IFactor::getTransferAllowanceCall {
                index_id: vault.index_id.get().to(),
                sender: owner,
                receiver: spender,
            },
        )?;
        Ok(U256::from(result._0))
    }

    pub fn approve(&mut self, spender: Address, value: U256) -> Result<bool, Vec<u8>> {
        if spender.is_zero() {
            Err(b"Invalid Spender")?;
        }
        let vault = Self::_storage();
        let result = self.external_call_ret(
            vault.gate_to_castle.get(),
            IFactor::approveTransferFromCall {
                index_id: vault.index_id.get().to(),
                receiver: spender,
                amount: value.to(),
            },
        )?;
        Ok(result._0)
    }

    pub fn transfer_from(
        &mut self,
        from: Address,
        to: Address,
        value: U256,
    ) -> Result<bool, Vec<u8>> {
        if from.is_zero() {
            Err(b"Invalid Spender")?;
        }
        if to.is_zero() {
            Err(b"Invalid Receiver")?;
        }
        let mut vault = Self::_storage();
        //spend_allowance(from, value)?;

        todo!("Transfer From");

        let event = IERC20::Transfer { from, to, value };

        self.vm().emit_log(&event.encode_data(), 1);
        Ok(true)
    }

    // UUPS

    #[selector(name = "UPGRADE_INTERFACE_VERSION")]
    fn upgrade_interface_version(&self) -> String {
        UPGRADE_INTERFACE_VERSION.into()
    }

    #[payable]
    pub fn upgrade_to_and_call(
        &mut self,
        new_implementation: Address,
        data: Bytes,
    ) -> Result<(), Vec<u8>> {
        let mut vault = Self::_storage();
        self._only_owner(&vault)?;
        todo!()
    }

    fn proxiable_uuid(&self) -> Result<B256, Vec<u8>> {
        todo!()
    }

    fn initialize(&mut self, owner: Address, asset: Address) -> Result<(), Vec<u8>> {
        let mut vault = Self::_storage();
        self._only_owner(&vault)?;
        self._set_version(&mut vault)?;
        self._set_collateral_asset(&mut vault, asset)?;
        self._transfer_ownership(&mut vault, owner)?;
        Ok(())
    }

    pub fn set_version(&mut self) -> Result<(), Vec<u8>> {
        let mut vault = Self::_storage();
        self._only_owner(&vault)?;
        self._set_version(&mut vault)
    }

    pub fn get_version(&self) -> U32 {
        VERSION_NUMBER
    }

    // IOwnable

    fn owner(&self) -> Address {
        let vault = Self::_storage();
        vault.owner.get()
    }

    fn transfer_ownership(&mut self, new_owner: Address) -> Result<(), Vec<u8>> {
        let mut vault = Self::_storage();
        self._only_owner(&vault)?;
        self._transfer_ownership(&mut vault, new_owner)
    }

    fn renounce_ownership(&mut self) -> Result<(), Vec<u8>> {
        let mut vault = Self::_storage();
        self._only_owner(&vault)?;
        self._renounce_ownership(&mut vault)
    }
}
