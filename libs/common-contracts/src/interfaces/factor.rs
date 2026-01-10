use alloy_sol_types::sol;

sol! {
    interface IFactor  {
        function submitMarketData(uint128 vendor_id, bytes calldata asset_names, bytes calldata asset_liquidity, bytes calldata asset_prices, bytes calldata asset_slopes) external;

        function submitBuyOrder(uint128 vendor_id, uint128 index_id, address trader_address, uint128 collateral_added, uint128 collateral_removed) external;

        function submitSellOrder(uint128 vendor_id, uint128 index_id, address trader_address, uint128 collateral_added, uint128 collateral_removed) external;

        function processPendingBuyOrder(uint128 vendor_id, uint128 index_id, address trader_address, uint128 max_order_size) external returns (bytes[] memory);

        function processPendingSellOrder(uint128 vendor_id, uint128 index_id, address trader_address, uint128 max_order_size) external returns (bytes[] memory);

        function executeBuyOrder(uint128 vendor_id, uint128 index_id, address trader_address, address operator_address, uint128 collateral_amount, uint128 max_order_size) external returns (bytes[] memory);

        function executeSellOrder(uint128 vendor_id, uint128 index_id, address trader_address, address operator_address, uint128 itp_amount, uint128 max_order_size) external returns (bytes[] memory);

        function executeTransfer(uint128 index_id, address sender, address receiver, uint128 amount) external;
    }
}
