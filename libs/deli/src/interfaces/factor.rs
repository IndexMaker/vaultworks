use alloy_sol_types::sol;

sol! {
    interface IFactor {
        function submitMarketData(uint128 vendor_id,
            uint8[] memory asset_names, 
            uint8[] memory asset_liquidity, 
            uint8[] memory asset_prices, 
            uint8[] memory asset_slopes) external;

        function updateIndexQuote(uint128 vendor_id, uint128 index) external;

        function updateMultipleIndexQuotes(
            uint128 vendor_id, 
            uint128[] memory indexes) external;

        function submitBuyOrder(
            uint128 vendor_id, 
            uint128 index, 
            uint128 collateral_amount) external;
    }
}