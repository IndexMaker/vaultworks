// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::{string::String, vec::Vec};

use alloy_primitives::{uint, Address, B256, U256, U32, U8};
use alloy_sol_types::{sol, SolEvent};
use common::{amount::Amount, vector::Vector};
use common_contracts::{
    contracts::{
        calls::InnerCall,
        formulas::{Order, Quote},
        keep_calls::KeepCalls,
        storage::StorageSlot,
        vault::{VAULT_STORAGE_SLOT, VaultStorage},
    },
    interfaces::factor::IFactor,
};
use stylus_sdk::{abi::Bytes, prelude::*};

pub const VERSION_NUMBER: U32 = uint!(1_U32);
pub const UPGRADE_INTERFACE_VERSION: &str = "5.0.0";

sol! {
    interface IERC20 {
        event Transfer(address indexed from, address indexed to, uint256 value);

        event Approval(address indexed owner, address indexed spender, uint256 value);

        function transferFrom(address from, address to, uint256 value) external returns (bool);
    }

    event Deposit(address indexed sender, address indexed owner, uint256 assets, uint256 shares);
    event Withdraw(address indexed sender, address indexed receiver, address indexed owner, uint256 assets, uint256 shares);

    event DepositRequest(address controller, address owner, uint256 requestId, address sender, uint256 assets);
    event RedeemRequest(address controller, address owner, uint256 requestId, address sender, uint256 shares);

    event OperatorSet(address controller, address operator, bool approved);
}

#[storage]
#[entrypoint]
pub struct Vault;

impl Vault {
    fn _storage() -> VaultStorage {
        StorageSlot::get_slot::<VaultStorage>(VAULT_STORAGE_SLOT)
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

        let itp_amount = Order::tell_total(bid, ask)?;
        let assets_base_value = Quote::tell_base_value(quote, itp_amount)?;

        Ok(assets_base_value.to_u256())
    }

    pub fn total_assets(&self) -> Result<U256, Vec<u8>> {
        let vault = Self::_storage();

        let (bid, ask) = self._get_total_order(&vault)?;
        let quote = self._get_quote(&vault)?;

        let itp_amount = Order::tell_total(bid, ask)?;
        let assets_base_value = Quote::tell_base_value(quote, itp_amount)?;

        Ok(assets_base_value.to_u256())
    }

    pub fn convert_to_assets(&self, shares: U256) -> Result<U256, Vec<u8>> {
        let vault = Self::_storage();

        let quote = self._get_quote(&vault)?;
        let itp_amount = Amount::try_from_u256(shares).ok_or_else(|| b"MathOverflow")?;
        let base_value = Quote::tell_base_value(quote, itp_amount)?;

        Ok(base_value.to_u256())
    }

    pub fn convert_to_shares(&self, assets: U256) -> Result<U256, Vec<u8>> {
        let vault = Self::_storage();

        let quote = self._get_quote(&vault)?;
        let base_value = Amount::try_from_u256(assets).ok_or_else(|| b"MathOverflow")?;
        let itp_amount = Quote::tell_itp_amount(quote, base_value)?;

        Ok(itp_amount.to_u256())
    }

    pub fn is_operator(&self, owner: Address, operator: Address) -> bool {
        let vault = Self::_storage();
        let operators = vault.operators.getter(owner);
        operators.is_operator(operator)
    }

    pub fn set_operator(&mut self, operator: Address, approved: bool) -> bool {
        let sender = self.attendee();
        let mut vault = Self::_storage();
        let mut operators = vault.operators.setter(sender);
        operators.set_operator(operator, approved);

        let event = OperatorSet {
            controller: sender,
            operator,
            approved,
        };

        self.vm().emit_log(&event.encode_data(), 1);
        true
    }

    pub fn request_deposit(
        &mut self,
        assets: U256,
        controller: Address,
        owner: Address,
    ) -> Result<U256, Vec<u8>> {
        if assets.is_zero() {
            Err(b"Shares cannot be zero")?;
        }
        let mut vault = Self::_storage();
        let sender = self.attendee();

        // Any user or their approved operator can send deposit request
        if sender != owner && !self.is_operator(owner, sender) {
            Err(b"Sender must be an owner or approved operator")?;
        }

        // Transfer USDC collateral from user to dedicated custody
        let asset = vault.collateral_asset.get();
        self.external_call_ret(
            asset,
            IERC20::transferFromCall {
                from: owner,
                to: vault.custody.get(),
                value: assets,
            },
        )?;

        // Requests from multiple users are aggregated per controller
        let mut request = vault.deposit_request.setter(controller);
        let request_id =
            request.request(Amount::try_from_u256(assets).ok_or_else(|| b"MathOverflow")?)?;

        // Send an event, and it will be picked up by Keeper service
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

    pub fn pending_deposit_request(
        &self,
        request_id: U256,
        controller: Address,
    ) -> Result<U256, Vec<u8>> {
        let vault = Self::_storage();
        let request = vault.deposit_request.getter(controller);
        if request.is_active() {
            Err(b"NotSuchRequest")?;
        }
        let amount = request.pending(request_id);
        Ok(amount.to_u256())
    }

    pub fn claimable_deposit_request(
        &self,
        request_id: U256,
        controller: Address,
    ) -> Result<U256, Vec<u8>> {
        let vault = Self::_storage();
        let request = vault.deposit_request.getter(controller);
        if request.is_active() {
            Err(b"NotSuchRequest")?;
        }
        let amount = request.claimable(request_id);
        Ok(amount.to_u256())
    }

    pub fn claimable_deposit_update(
        &self,
        request_id: U256,
        assets: U256,
        shares: U256,
    ) -> Result<(), Vec<u8>> {
        let mut vault = Self::_storage();
        let mut request = vault.deposit_request.setter(self.attendee());
        if request.is_active() {
            Err(b"NotSuchRequest")?;
        }
        request.update(
            request_id,
            Amount::try_from_u256(assets).ok_or_else(|| b"MathOverflow")?,
            Amount::try_from_u256(shares).ok_or_else(|| b"MathOverflow")?,
        )?;
        Ok(())
    }

    pub fn deposit(
        &mut self,
        assets: U256,
        receiver: Address,
        controller: Address,
    ) -> Result<U256, Vec<u8>> {
        let mut vault = Self::_storage();
        let sender = self.attendee();

        // User can claim their shares or controller can claim for the user or
        // approved operator.
        if sender != receiver
            && sender != controller
            && !self.is_operator(receiver, sender)
            && !self.is_operator(controller, sender)
        {
            Err(b"Sender must be an owner or approved operator")?;
        }

        let mut request = vault.deposit_request.setter(controller);
        if request.is_active() {
            Err(b"NotSuchRequest")?;
        }
        let shares =
            request.claim(Amount::try_from_u256(assets).ok_or_else(|| b"MathOverflow")?)?;

        let event = Deposit {
            sender: controller,
            owner: receiver,
            assets,
            shares: shares.to_u256(),
        };

        self.vm().emit_log(&event.encode_data(), 1);

        Ok(shares.to_u256())
    }

    pub fn request_redeem(
        &mut self,
        shares: U256,
        controller: Address,
        owner: Address,
    ) -> Result<U256, Vec<u8>> {
        if shares.is_zero() {
            Err(b"Shares cannot be zero")?;
        }

        let mut vault = Self::_storage();
        let sender = self.attendee();

        // Any user or their approved operator can send redeem request
        if sender != owner && !self.is_operator(owner, sender) {
            Err(b"Sender must be an owner or approved operator")?;
        }

        // Requests from multiple users are aggregated per controller
        let mut request = vault.redeem_request.setter(controller);
        let request_id =
            request.request(Amount::try_from_u256(shares).ok_or_else(|| b"MathOverflow")?)?;

        // Send an event, and it will be picked up by Keeper service
        let request_event = RedeemRequest {
            controller,
            owner,
            requestId: request_id,
            sender: self.attendee(),
            shares,
        };

        self.vm().emit_log(&request_event.encode_data(), 1);

        Ok(request_id)
    }

    pub fn pending_redeem_request(
        &self,
        request_id: U256,
        controller: Address,
    ) -> Result<U256, Vec<u8>> {
        let vault = Self::_storage();
        let request = vault.redeem_request.getter(controller);
        if request.is_active() {
            Err(b"NotSuchRequest")?;
        }
        let amount = request.pending(request_id);
        Ok(amount.to_u256())
    }

    pub fn claimable_redeem_request(
        &self,
        request_id: U256,
        controller: Address,
    ) -> Result<U256, Vec<u8>> {
        let vault = Self::_storage();
        let request = vault.redeem_request.getter(controller);
        if request.is_active() {
            Err(b"NotSuchRequest")?;
        }
        let amount = request.claimable(request_id);
        Ok(amount.to_u256())
    }

    pub fn claimable_redeem_update(
        &self,
        request_id: U256,
        shares: U256,
        assets: U256,
    ) -> Result<(), Vec<u8>> {
        let mut vault = Self::_storage();
        let mut request = vault.redeem_request.setter(self.attendee());
        if request.is_active() {
            Err(b"NotSuchRequest")?;
        }
        request.update(
            request_id,
            Amount::try_from_u256(shares).ok_or_else(|| b"MathOverflow")?,
            Amount::try_from_u256(assets).ok_or_else(|| b"MathOverflow")?,
        )?;
        Ok(())
    }

    pub fn redeem(
        &mut self,
        shares: U256,
        receiver: Address,
        controller: Address,
    ) -> Result<U256, Vec<u8>> {
        let mut vault = Self::_storage();
        let sender = self.attendee();

        // User can claim their USDC or controller can claim for the user or
        // approved operator.
        if sender != receiver
            && sender != controller
            && !self.is_operator(receiver, sender)
            && !self.is_operator(controller, sender)
        {
            Err(b"Sender must be an owner or approved operator")?;
        }

        let mut request = vault.redeem_request.setter(controller);
        if request.is_active() {
            Err(b"NotSuchRequest")?;
        }
        let assets =
            request.claim(Amount::try_from_u256(shares).ok_or_else(|| b"MathOverflow")?)?;

        // Transfer USDC collateral from dedicated custody to the user
        let asset = vault.collateral_asset.get();
        self.external_call_ret(
            asset,
            IERC20::transferFromCall {
                from: vault.custody.get(),
                to: receiver,
                value: assets.to_u256(),
            },
        )?;

        let event = Withdraw {
            sender,
            receiver,
            owner: controller,
            assets: assets.to_u256(),
            shares,
        };

        self.vm().emit_log(&event.encode_data(), 1);

        Ok(assets.to_u256())
    }

    // fn max_deposit(&self, receiver: Address) -> U256 {
    //     U256::ZERO
    // }

    // fn preview_deposit(&self, assets: U256) -> Result<U256, Vec<u8>> {
    //     Err(b"Must deposit via Keeper service".into())
    // }

    // fn deposit(&mut self, assets: U256, receiver: Address) -> Result<U256, Vec<u8>> {
    //     Err(b"Must deposit via Keeper service".into())
    // }

    // fn max_redeem(&self, owner: Address) -> U256 {
    //     U256::ZERO
    // }

    // fn preview_redeem(&self, shares: U256) -> Result<U256, Vec<u8>> {
    //     Err(b"Must redeem via Keeper service".into())
    // }

    // fn redeem(&mut self, shares: U256, receiver: Address, owner: Address) -> Result<U256, Vec<u8>> {
    //     Err(b"Must redeem via Keeper service".into())
    // }

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

        let (bid, ask) = self._get_total_order(&vault)?;
        let itp_amount = Order::tell_total(bid, ask)?;

        Ok(itp_amount.to_u256())
    }

    pub fn balance_of(&self, account: Address) -> Result<U256, Vec<u8>> {
        let vault = Self::_storage();

        let (bid, ask) = self._get_order(&vault, account)?;
        let itp_amount = Order::tell_available(bid, ask)?;

        Ok(itp_amount.to_u256())
    }

    pub fn transfer(&mut self, to: Address, value: U256) -> Result<(), Vec<u8>> {
        let vault = Self::_storage();
        let sender = self.attendee();

        // Vault is submitting transfer on behalf of msg.sender (attendee)
        self.external_call(
            vault.gate_to_castle.get(),
            IFactor::submitTransferCall {
                index_id: vault.index_id.get().to(),
                sender,
                receiver: to,
                amount: Amount::try_from_u256(value)
                    .ok_or_else(|| b"MathOverflow")?
                    .to_u128_raw(),
            },
        )?;

        let event = IERC20::Transfer {
            from: sender,
            to,
            value,
        };

        self.vm().emit_log(&event.encode_data(), 1);

        Ok(())
    }

    pub fn allowance(&self, owner: Address, spender: Address) -> U256 {
        let vault = Self::_storage();
        let allowances = vault.allowances.get(owner);
        allowances.allowance(spender)
    }

    pub fn approve(&mut self, spender: Address, value: U256) -> Result<bool, Vec<u8>> {
        if spender.is_zero() {
            Err(b"Invalid Spender")?;
        }
        let mut vault = Self::_storage();
        let mut allowance = vault.allowances.setter(self.attendee());
        allowance.approve(spender, value)
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
        let mut allowance = vault.allowances.setter(self.attendee());
        allowance.spend_allowance(from, value)?;

        self.external_call(
            vault.gate_to_castle.get(),
            IFactor::submitTransferCall {
                index_id: vault.index_id.get().to(),
                sender: from,
                receiver: to,
                amount: Amount::try_from_u256(value)
                    .ok_or_else(|| b"MathOverflow")?
                    .to_u128_raw(),
            },
        )?;

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
