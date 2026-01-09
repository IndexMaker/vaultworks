use alloy_sol_types::sol;

sol! {
    interface IFactor  {
        function submitMarketData(uint128 vendor_id, bytes calldata asset_names, bytes calldata asset_liquidity, bytes calldata asset_prices, bytes calldata asset_slopes) external;

        function processTraderBuyOrder(uint128 vendor_id, uint128 index_id, address trader_address, uint128 max_order_size) external returns (bytes memory, bytes memory, bytes memory);

        function processTraderSellOrder(uint128 vendor_id, uint128 index_id, address trader_address, uint128 max_order_size) external returns (bytes memory, bytes memory, bytes memory);

        function submitBuyOrder(uint128 vendor_id, uint128 index_id, address trader_address, uint128 collateral_added, uint128 collateral_removed) external;

        function submitSellOrder(uint128 vendor_id, uint128 index_id, address trader_address, uint128 collateral_added, uint128 collateral_removed) external;

        function executeBuyOrder(uint128 vendor_id, uint128 index_id, address trader_address, uint128 collateral_added, uint128 collateral_removed, uint128 max_order_size) external returns (bytes memory, bytes memory, bytes memory);

        function executeSellOrder(uint128 vendor_id, uint128 index_id, address trader_address, uint128 collateral_added, uint128 collateral_removed, uint128 max_order_size) external returns (bytes memory, bytes memory, bytes memory);

        function executeTransfer(uint128 index_id, address sender, address receiver, uint128 amount) external;
    }
}
