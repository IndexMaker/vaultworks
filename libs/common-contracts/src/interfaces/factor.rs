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
            uint128 collateral_added, 
            uint128 collateral_removed, 
            uint128 max_order_size, uint8[] 
            memory asset_contribution_fractions
        ) external returns (
            uint8[] memory,     // index order
            uint8[] memory,     // index order executed / remaining
            uint8[] memory);    // executed asset quantities

        function fetchMarketData(uint128 vendor_id) external view returns (uint8[] memory, uint8[] memory, uint8[] memory);

        function fetchIndexQuote(uint128 index) external view returns (uint8[] memory);

        function getOrderCount(uint128 index) external view returns (uint128);

        function getOrder(uint128 index, uint128 offset) external view returns (address, uint8[] memory);
    }
}
