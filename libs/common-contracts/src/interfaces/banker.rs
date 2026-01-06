use alloy_sol_types::sol;

sol! {
    interface IBanker  {
        function submitAssets(uint128 vendor_id, bytes calldata market_asset_names) external;

        function submitMargin(uint128 vendor_id, bytes calldata asset_names, bytes calldata asset_margin) external;

        function submitSupply(uint128 vendor_id, bytes calldata asset_names, bytes calldata asset_quantities_short, bytes calldata asset_quantities_long) external;

        function getVendorAssets(uint128 vendor_id) external returns (uint8[] memory, uint8[] memory);

        function getVendorMargin(uint128 vendor_id) external returns (uint8[] memory);

        function getVendorSupply(uint128 vendor_id) external returns (uint8[] memory, uint8[] memory);

        function getVendorDemand(uint128 vendor_id) external returns (uint8[] memory, uint8[] memory);

        function getVendorDelta(uint128 vendor_id) external returns (uint8[] memory, uint8[] memory);
    }
}