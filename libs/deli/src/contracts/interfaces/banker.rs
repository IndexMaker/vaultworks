use alloy_sol_types::sol;

sol! {
    interface IBanker  {
        function submitAssets(uint128 vendor_id, uint8[] memory market_asset_names) external;

        function submitMargin(uint128 vendor_id, uint8[] memory asset_names, uint8[] memory asset_margin) external;

        function submitSupply(uint128 vendor_id, uint8[] memory asset_names, uint8[] memory asset_quantities_short, uint8[] memory asset_quantities_long) external;
    }
}