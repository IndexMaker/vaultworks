use alloy_sol_types::sol;

sol! {
    interface ISteward  {
        function getVault(uint128 index_id) external view returns (address);

        function getMarketData(uint128 vendor_id) external view returns (bytes[] memory);

        function getIndexAssetsCount(uint128 index_id) external view returns (uint128);

        function getIndexAssets(uint128 index_id) external view returns (bytes memory);

        function getIndexWeights(uint128 index_id) external view returns (bytes memory);

        function getIndexQuote(uint128 index_id, uint128 vendor_id) external view returns (bytes memory);

        function getTraderOrder(uint128 index_id, address trader) external view returns (bytes memory);

        function getTraderCount(uint128 index_id) external view returns (uint128);

        function getTraderAt(uint128 index_id, uint128 offset) external view returns (address);

        function getVendorOrder(uint128 index_id, uint128 vendor_id) external view returns (bytes memory);

        function getVendorCount(uint128 index_id) external view returns (uint128);

        function getVendorAt(uint128 index_id, uint128 offset) external view returns (uint128);

        function getTotalOrder(uint128 index_id) external view returns (bytes memory);

        function getVendorAssets(uint128 vendor_id) external returns (bytes memory);

        function getVendorMargin(uint128 vendor_id) external returns (bytes memory);

        function getVendorSupply(uint128 vendor_id) external returns (bytes[] memory);

        function getVendorDemand(uint128 vendor_id) external returns (bytes[] memory);

        function getVendorDelta(uint128 vendor_id) external returns (bytes[] memory);

        function fetchVector(uint128 id) external view returns (bytes memory);
    }
}