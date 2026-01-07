// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U128, U256};
use alloy_sol_types::{sol, SolEvent};
use common::amount::Amount;
use common_contracts::{
    contracts::{
        calls::InnerCall,
        keep_calls::KeepCalls,
        vault::VaultStorage, vault_requests::VaultRequestsStorage,
    },
    interfaces::factor::IFactor,
};
use stylus_sdk::prelude::*;

sol! {
    interface IERC20 {
        function transferFrom(address from, address to, uint256 value) external returns (bool);
    }

    event DepositRequest(address controller, address owner, uint256 requestId, address sender, uint256 assets);
    event Deposit(address indexed sender, address indexed owner, uint256 assets, uint256 shares);


    event RedeemRequest(address controller, address owner, uint256 requestId, address sender, uint256 shares);
    event Withdraw(address indexed sender, address indexed receiver, address indexed owner, uint256 assets, uint256 shares);


    event OperatorSet(address controller, address operator, bool approved);
}

#[storage]
#[entrypoint]
pub struct VaultRequests;

#[public]
impl VaultRequests {
    pub fn configure_requests(
        &mut self,
        vendor_id: U128,
        custody: Address,
        asset: Address,
    ) -> Result<(), Vec<u8>> {
        let vault = VaultStorage::storage();
        vault.only_owner(self.attendee())?;
        
        let mut requests = VaultRequestsStorage::storage();
        requests.vendor_id.set(vendor_id);
        requests.custody.set(custody);
        requests.collateral_asset.set(asset);

        Ok(())
    }

    // ERC4626

    fn asset(&self) -> Address {
        let requests = VaultRequestsStorage::storage();
        requests.collateral_asset.get()
    }

    pub fn assets(&self, account: Address) -> Result<U256, Vec<u8>> {
        let vault = VaultStorage::storage();
        let requests = VaultRequestsStorage::storage();

        let order = vault.get_order(self, account)?;
        let quote = requests.get_quote(&vault, self)?;

        let itp_amount = order.tell_total()?;
        let assets_base_value = quote.tell_base_value(itp_amount)?;

        Ok(assets_base_value.to_u256())
    }

    pub fn total_assets(&self) -> Result<U256, Vec<u8>> {
        let vault = VaultStorage::storage();
        let requests = VaultRequestsStorage::storage();

        let order = vault.get_total_order(self)?;
        let quote = requests.get_quote(&vault, self)?;

        let itp_amount = order.tell_total()?;
        let assets_base_value = quote.tell_base_value(itp_amount)?;

        Ok(assets_base_value.to_u256())
    }

    pub fn convert_to_assets(&self, shares: U256) -> Result<U256, Vec<u8>> {
        let vault = VaultStorage::storage();
        let requests = VaultRequestsStorage::storage();

        let quote = requests.get_quote(&vault, self)?;
        let itp_amount = Amount::try_from_u256(shares).ok_or_else(|| b"MathOverflow")?;
        let base_value = quote.tell_base_value(itp_amount)?;

        Ok(base_value.to_u256())
    }

    pub fn convert_to_shares(&self, assets: U256) -> Result<U256, Vec<u8>> {
        let vault = VaultStorage::storage();
        let requests = VaultRequestsStorage::storage();

        let quote = requests.get_quote(&vault, self)?;
        let base_value = Amount::try_from_u256(assets).ok_or_else(|| b"MathOverflow")?;
        let itp_amount = quote.tell_itp_amount(base_value)?;

        Ok(itp_amount.to_u256())
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

    // ERC-7540

    pub fn is_operator(&self, owner: Address, operator: Address) -> bool {
        let requests = VaultRequestsStorage::storage();
        let operators = requests.operators.getter(owner);
        operators.is_operator(operator)
    }

    pub fn set_operator(&mut self, operator: Address, approved: bool) -> bool {
        let sender = self.attendee();
        let mut requests = VaultRequestsStorage::storage();
        let mut operators = requests.operators.setter(sender);
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
        let mut requests = VaultRequestsStorage::storage();
        let sender = self.attendee();

        // Any user or their approved operator can send deposit request
        if sender != owner && !self.is_operator(owner, sender) {
            Err(b"Sender must be an owner or approved operator")?;
        }

        // Transfer USDC collateral from user to dedicated custody
        let asset = requests.collateral_asset.get();
        self.external_call(
            asset,
            IERC20::transferFromCall {
                from: owner,
                to: requests.custody.get(),
                value: assets,
            },
        )?;

        // Requests from multiple users are aggregated per controller
        let mut request = requests.deposit_request.setter(controller);
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
        let requests = VaultRequestsStorage::storage();
        let request = requests.deposit_request.getter(controller);
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
        let requests = VaultRequestsStorage::storage();
        let request = requests.deposit_request.getter(controller);
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
        let mut requests = VaultRequestsStorage::storage();
        let mut request = requests.deposit_request.setter(self.attendee());
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
        let vault = VaultStorage::storage();
        let mut requests = VaultRequestsStorage::storage();
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

        let mut request = requests.deposit_request.setter(controller);
        if request.is_active() {
            Err(b"NotSuchRequest")?;
        }
        let shares =
            request.claim(Amount::try_from_u256(assets).ok_or_else(|| b"MathOverflow")?)?;

        // Transfer ITP from Keeper account to Trader account
        self.external_call(
            vault.gate_to_castle.get(),
            IFactor::submitTransferCall {
                index_id: vault.index_id.get().to(),
                sender: controller,
                receiver,
                amount: shares.to_u128_raw(),
            },
        )?;

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

        let vault = VaultStorage::storage();
        let mut requests = VaultRequestsStorage::storage();
        let sender = self.attendee();

        // Any user or their approved operator can send redeem request
        if sender != owner && !self.is_operator(owner, sender) {
            Err(b"Sender must be an owner or approved operator")?;
        }

        // Transfer ITP from Trader account to Keeper account
        self.external_call(
            vault.gate_to_castle.get(),
            IFactor::submitTransferCall {
                index_id: vault.index_id.get().to(),
                sender: owner,
                receiver: controller,
                amount: shares.to(),
            },
        )?;

        // Requests from multiple users are aggregated per controller
        let mut request = requests.redeem_request.setter(controller);
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
        let requests = VaultRequestsStorage::storage();
        let request = requests.redeem_request.getter(controller);
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
        let requests = VaultRequestsStorage::storage();
        let request = requests.redeem_request.getter(controller);
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
        let mut requests = VaultRequestsStorage::storage();
        let mut request = requests.redeem_request.setter(self.attendee());
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
        let mut requests = VaultRequestsStorage::storage();
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

        let mut request = requests.redeem_request.setter(controller);
        if request.is_active() {
            Err(b"NotSuchRequest")?;
        }
        let assets =
            request.claim(Amount::try_from_u256(shares).ok_or_else(|| b"MathOverflow")?)?;

        // Transfer USDC collateral from dedicated custody to the user
        let asset = requests.collateral_asset.get();
        self.external_call(
            asset,
            IERC20::transferFromCall {
                from: requests.custody.get(),
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
}
