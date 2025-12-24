use alloy_sol_types::sol;

sol! {
    interface IFactor  {
        function submitMarketData(uint128 vendor_id, uint8[] memory asset_names, uint8[] memory asset_liquidity, uint8[] memory asset_prices, uint8[] memory asset_slopes) external;

        function updateIndexQuote(uint128 vendor_id, uint128 index_id) external;

        function updateMultipleIndexQuotes(uint128 vendor_id, uint128[] memory index_ids) external;

        function submitBuyOrder(uint128 vendor_id, uint128 index_id, uint128 collateral_added, uint128 collateral_removed, uint128 max_order_size, uint8[] memory asset_contribution_fractions) external returns (uint8[] memory, uint8[] memory, uint8[] memory);

        function submitSellOrder(uint128 vendor_id, uint128 index_id, uint128 itp_added, uint128 itp_removed, uint128 max_order_size, uint8[] memory asset_contribution_fractions) external returns (uint8[] memory, uint8[] memory, uint8[] memory);

        function submitRebalanceOrder(uint128 vendor_id, uint8[] memory new_assets, uint8[] memory new_weigthts) external;

        function getMarketData(uint128 vendor_id) external view returns (uint8[] memory, uint8[] memory, uint8[] memory);

        function getIndexAssets(uint128 index_id) external view returns (uint8[] memory);

        function getIndexWeights(uint128 index_id) external view returns (uint8[] memory);

        function getIndexQuote(uint128 index_id, uint128 vendor_id) external view returns (uint8[] memory);

        function getTraderOrder(uint128 index_id, address trader) external view returns (uint8[] memory, uint8[] memory);

        function getTraderCount(uint128 index_id) external view returns (uint128);

        function getTraderOrderAt(uint128 index_id, uint128 offset) external view returns (address, uint8[] memory, uint8[] memory);

        function getVendorOrder(uint128 index_id, uint128 vendor_id) external view returns (uint8[] memory, uint8[] memory);

        function getVendorCount(uint128 index_id) external view returns (uint128);

        function getVendorOrderAt(uint128 index_id, uint128 offset) external view returns (uint128, uint8[] memory, uint8[] memory);

        function getTotalOrder(uint128 index_id) external view returns (uint8[] memory, uint8[] memory);
    }
}
