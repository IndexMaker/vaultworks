use alloy_sol_types::sol;

sol! {
    interface IFactor {
        function submitMarketData(uint8[] memory _asset_names, uint8[] memory _asset_liquidity, uint8[] memory _asset_prices, uint8[] memory _asset_slopes) external;

        function updateIndexQuote(uint128 index) external;

        function updateMultipleIndexQuotes(uint128[] memory indexes) external;

        function submitBuyOrder(uint128 index, uint128 collateral_amount) external;
    }
}