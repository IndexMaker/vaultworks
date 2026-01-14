use alloy_sol_types::sol;

sol! {
    interface IBanker  {
        function submitAssets(uint128 vendor_id, bytes calldata market_asset_names) external;

        function submitMargin(uint128 vendor_id, bytes calldata asset_names, bytes calldata asset_margin) external;

        function submitSupply(uint128 vendor_id, bytes calldata asset_names, bytes calldata asset_quantities_short, bytes calldata asset_quantities_long) external;

        function submitMarketData(uint128 vendor_id, bytes calldata asset_names, bytes calldata asset_liquidity, bytes calldata asset_prices, bytes calldata asset_slopes) external;
        
        function updateIndexQuote(uint128 vendor_id, uint128 index_id) external;

        function updateMultipleIndexQuotes(uint128 vendor_id, uint128[] memory index_ids) external;

        event IndexQuoteUpdated(uint128 index_id, address sender);
    }
}
