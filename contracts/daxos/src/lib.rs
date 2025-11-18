// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;

use alloy_primitives::{Address, U128};
use alloy_sol_types::{sol, SolCall};
use deli::labels::Labels;
use icore::vil::{
    execute_buy_order::execute_buy_order, update_margin::update_margin,
    update_market_data::update_market_data, update_quote::update_quote,
    update_supply::update_supply,
};
use stylus_sdk::{
    prelude::*,
    storage::{StorageAddress, StorageMap},
};

sol! {
    /// Vector IL (VIL) virtual machine
    ///
    /// Performs operations on vectors stored on-chain as opaque blobs.  By
    /// using dedicated VIL for vector processing we save on (de)serialisation
    /// of blobs and also on SLOAD/SSTORE operations, because we have all vector
    /// operations integrated with storage of vectors as the blobs, meaning that
    /// we can submit VIL program that will perform number of vector
    /// instructions on vectors using only one SLOAD for each vector load, and
    /// one SSTORE, as well as we don't need to SSTORE intermediate results as
    /// they are stored on internal stack of the virtual machine.
    interface IDevil  {
        function setup(address owner) external;

        function submit(uint128 id, uint8[] memory data) external;

        function get(uint128 id) external view returns (uint8[] memory);

        function execute(uint8[] memory code, uint128 num_registry) external;
    }

    /// Market monitors supply and demand for assets
    ///
    /// Vault orders update demand, while authorised provider updates supply.
    /// The delta monitors difference between suppy and demand, and is critical
    /// metric for:
    ///     a) authorised provider to know which assets to buy/sell
    ///     b) daxos to match new orders or halt (throttle order over time)
    ///
    /// All data is stored as vectors on DeVIL virtual machine, and Market
    /// itself only organises handles to those vectors and submits VIL programs
    /// to execute. The results of those programs executions stay on DeVIL, but
    /// can be accessed when required by calling Devil::get(vector_id) method.
    ///
    interface IMarket  {
        function setup(address owner, address devil) external;

        function submitSupply() external;

        function getSupply() external view returns (uint128, uint128);

        function getDemand() external view returns (uint128, uint128);

        function getDelta() external view returns (uint128, uint128);

        function getLiquidity() external view returns (uint128);

        function getPrices() external view returns (uint128);

        function getSlopes() external view returns (uint128);
    }

    /// Vault (a.k.a. Index) tracks its price and orders
    ///
    /// Vault stores:
    ///     - asset weights
    ///     - latest quote, which consists of: Capacity, Price, and Slope (Price
    ///     change with quantity)
    ///     - order queue
    ///
    /// All data is stored as vectors on DeVIL virtual machine, and Vault itself
    /// only organises handles to those vectors and submits VIL programs to
    /// execute.
    interface IVault  {
        function setup(address owner, address devil) external;

        function submitOrder(address user, uint128 collateral_amount) external;

        function getQueue() external view returns (uint128);

        function getAssets() external view returns (uint128);

        function getWeights() external view returns (uint128);

        function getQuote() external view returns (uint128);
    }
}

#[storage]
#[entrypoint]
pub struct Daxos {
    owner: StorageAddress,
    devil: StorageAddress,
    market: StorageAddress,
    vaults: StorageMap<U128, StorageAddress>,
}

impl Daxos {
    fn check_owner(&self, address: Address) -> Result<(), Vec<u8>> {
        let current_owner = self.owner.get();
        if !current_owner.is_zero() && address != current_owner {
            Err(b"Mut be owner")?;
        }
        Ok(())
    }

    fn send_to_devil(&mut self, code: Vec<u128>, num_registry: u128) -> Result<(), Vec<u8>> {
        let devil_call = IDevil::executeCall {
            code: Labels { data: code }.to_vec(),
            num_registry,
        };
        self.vm()
            .call(&self, self.devil.get(), &devil_call.abi_encode())?;
        Ok(())
    }

    /// Issuer has deployed Vault contract and now we need to set it up
    fn setup_vault(
        &mut self,
        vault_id: U128,
        vault_address: Address, /* ... setup params ...*/
    ) -> Result<(), Vec<u8>> {
        self.check_owner(self.vm().tx_origin())?;
        let mut vault_access = self.vaults.setter(vault_id);
        if !vault_access.get().is_zero() {
            Err(b"Duplicate Vault")?;
        }
        vault_access.set(vault_address);
        let me = self.vm().contract_address();
        let devil_address = self.devil.get();
        let vault_setup = IVault::setupCall {
            owner: me,
            devil: devil_address,
            /* ...setup params... */
        };
        self.vm()
            .call(&self, vault_address, &vault_setup.abi_encode())?;
        Ok(())
    }
}

#[public]
impl Daxos {
    /// Setup Daxos to use specific DeVIL and Market contracts
    pub fn setup(
        &mut self,
        owner: Address,
        devil: Address,
        market: Address,
    ) -> Result<(), Vec<u8>> {
        self.check_owner(self.vm().tx_origin())?;
        self.owner.set(owner);
        self.devil.set(devil);
        self.market.set(market);
        // TODO: send to devil solve_quadratic()
        Ok(())
    }

    /// Submit new Index
    ///
    /// Deploys Vault contract in inactive state. Needs to be voted to activate.
    ///
    pub fn submit_index(
        &mut self,
        index: U128,
        asset_names: Vec<u8>,
        asset_weights: Vec<u8>,
        info: Vec<u8>,
    ) -> Result<(), Vec<u8>> {
        // Note: `info` contrains all the information about the Index in binary format.
        // TODO: Find out what is the information and whether EVM format is more suitable.
        self.setup_vault(index, Address::ZERO)?;
        Ok(())
    }

    /// Submit a vote for an Index
    ///
    /// Once enough votes, Vault contract is activated.
    ///
    pub fn submit_vote(&mut self, index: U128, vote: Vec<u8>) -> Result<(), Vec<u8>> {
        // Should call Vault smart-contract method to vote on the Index.
        Ok(())
    }

    /// Submit BUY Index order
    ///
    /// Add collateral amount to user's order, and match for immediate execution.
    ///
    pub fn submit_buy_order(
        &mut self,
        index: U128,
        collateral_amount: u128,
    ) -> Result<(), Vec<u8>> {
        let user = self.vm().msg_sender();
        let vault_access = self.vaults.getter(index);
        let vault_address = vault_access.get();
        if vault_address.is_zero() {
            Err(b"Vault Not Found")?;
        }
        let submit = IVault::submitOrderCall {
            user,
            collateral_amount,
        };
        self.vm().call(&self, vault_address, &submit.abi_encode())?;

        // TODO: We need to set these up. They are from Vault and Market.
        let index_order_id = 10001;
        let executed_asset_quantities_id = 10002;
        let executed_index_quantities_id = 10003;
        let asset_names_id = 1001;
        let weights_id = 1002;
        let quote_id = 1003;
        let market_asset_names_id = 101;
        let supply_long_id = 102;
        let supply_short_id = 103;
        let demand_long_id = 104;
        let demand_short_id = 105;
        let delta_long_id = 106;
        let delta_short_id = 107;
        let margin_id = 999;
        let solve_quadratic_id = 10;

        // Compile VIL program, which we will send to DeVIL for execution
        //
        // The program:
        //  - updates index's quote, i.e. capacity, price, slope
        //
        // Note it could be a stored procedure as program is constant for each Vault.
        //
        let [asset_prices_id, asset_slopes_id, asset_liquidity_id] = [0; 3];
        let update = update_quote(
            asset_names_id,
            weights_id,
            quote_id,
            market_asset_names_id,
            asset_prices_id,
            asset_slopes_id,
            asset_liquidity_id,
            delta_long_id,
            delta_short_id,
            margin_id,
        );
        let num_registry = 16;
        self.send_to_devil(update, num_registry)?;

        // Compile VIL program, which we will send to DeVIL for execution.
        //
        // The program:
        //  - updates user's order with new collateral
        //  - executes portion of the order that fits within Index capacity
        //  - updates demand and delta vectors
        //  - returns amount of collateral remaining and spent, and
        //  - Index quantity executed and remaining
        //
        let update = execute_buy_order(
            index_order_id,
            collateral_amount,
            0,
            executed_index_quantities_id,
            executed_asset_quantities_id,
            asset_names_id,
            weights_id,
            quote_id,
            market_asset_names_id,
            supply_long_id,
            supply_short_id,
            demand_long_id,
            demand_short_id,
            delta_long_id,
            delta_short_id,
            margin_id,
            solve_quadratic_id,
        );
        let num_registry = 16;
        self.send_to_devil(update, num_registry)?;

        // TODO: Fetch results
        // - executed and remaining Index quantity
        // - collateral remaining and spent
        // - mint token if fully executed

        Ok(())
    }

    /// Submit supply
    ///
    /// Vendor submits new supply of assets. This new supply is an absolute
    /// quantity of assets and not delta.  However the supply is a sub-set of
    /// all assets stored in supply vector, so that Vendor does not need to send
    /// whole supply all the time, and only quantities of assets that have
    /// changed, e.g. as a result of fill. Vendor would accumulate fills over
    /// time period so that it doesn't call submit_supply() too often to save on
    /// gas, and in that time period Vendor would accumulate several fills for
    /// various assets, and absolute quantities of those assets after applying
    /// those fills would be submitted.
    ///
    /// Note that it is Vendor deciding how much of their internal inventory
    /// they are exposing to our transactions.
    ///
    pub fn submit_supply(
        &mut self,
        _asset_names: Vec<u8>,
        _asset_quantities_short: Vec<u8>,
        _asset_quantities_long: Vec<u8>,
    ) -> Result<(), Vec<u8>> {
        let market_address = self.market.get();
        let submit = IMarket::submitSupplyCall {};
        self.vm()
            .call(&self, market_address, &submit.abi_encode())?;

        // TODO: get those from Market
        let [asset_names_id, asset_quantities_short_id, asset_quantities_long_id] = [0; 3];
        let [market_asset_names_id, supply_long_id, supply_short_id] = [0; 3];
        let [demand_long_id, demand_short_id, delta_long_id, delta_short_id] = [0; 4];

        // Compile VIL program, which we will send to DeVIL for execution.
        //
        // The program:
        // - updates supply long and short by overwriting with supplied values
        // - computes delta long and short
        //
        let update = update_supply(
            asset_names_id,
            asset_quantities_short_id,
            asset_quantities_long_id,
            market_asset_names_id,
            supply_long_id,
            supply_short_id,
            demand_long_id,
            demand_short_id,
            delta_long_id,
            delta_short_id,
        );
        let num_registry = 16;
        self.send_to_devil(update, num_registry)?;
        Ok(())
    }

    /// Submit Market Data
    ///
    /// Vendor submits Market Data using Price, Slope, Liquidity model, which is
    /// a format optimised for on-chain computation.
    ///
    /// - Price     : Micro-Price
    /// - Slope     : Price delta within N-levels (Bid + Ask)
    /// - Liquidity : Total quantitiy on N-levels (Bid + Ask)
    ///
    /// Vendor is responsible for modeling these parameters is suitable way
    /// using live Market Data.
    ///
    /// Note that it is the Vendor deciding what prices and exposure they are
    /// willing to accept, i.e. they can adjust prices, slopes and liquidity to
    /// take into account their risk factors.
    ///
    pub fn submit_market_data(
        &mut self,
        _asset_names: Vec<u8>,
        _asset_liquidity: Vec<u8>,
        _asset_prices: Vec<u8>,
        _asset_slopes: Vec<u8>,
    ) -> Result<(), Vec<u8>> {
        let [asset_names_id, asset_prices_id, asset_slopes_id, asset_liquidity_id] = [0; 4];
        let [market_asset_names_id, market_asset_prices_id, market_asset_slopes_id, market_asset_liquidity_id] =
            [0; 4];

        // Compile VIL program, which we will send to DeVIL for execution.
        let update = update_market_data(
            asset_names_id,
            asset_prices_id,
            asset_slopes_id,
            asset_liquidity_id,
            market_asset_names_id,
            market_asset_prices_id,
            market_asset_slopes_id,
            market_asset_liquidity_id,
        );
        let num_registry = 16;
        self.send_to_devil(update, num_registry)?;
        Ok(())
    }

    /// Submit Margin
    /// 
    /// Vendor submits Margin, which limits how much of each asset we can
    /// allocate to new Index orders.
    /// 
    /// Asset Capacity = MIN(Market Liquidity, Margin - MAX(Demand Short, Demand Long))
    /// 
    /// Index Capacity = VMIN(Asset Capacity / Asset Weight)
    /// 
    pub fn submit_margin(
        &mut self,
        _asset_names: Vec<u8>,
        _asset_margin: Vec<u8>,
    ) -> Result<(), Vec<u8>> {
        // TODO: get those from Market
        let [asset_names_id, asset_margin_id, market_asset_names_id, margin_id] = [0; 4];

        // Compile VIL program, which we will send to DeVIL for execution.
        //
        // The program:
        // - updates maring by overwriting with supplied values
        //
        let update = update_margin(
            asset_names_id,
            asset_margin_id,
            market_asset_names_id,
            margin_id,
        );
        let num_registry = 16;
        self.send_to_devil(update, num_registry)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {}
