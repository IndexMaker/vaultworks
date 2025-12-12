use alloy_sol_types::sol;

sol! {
    interface IBanker {
        function submitAssets(uint8[] memory market_asset_names) external;

        function submitMargin(uint8[] memory _asset_names, uint8[] memory _asset_margin) external;

        function submitSupply(uint8[] memory _asset_names, uint8[] memory _asset_quantities_short, uint8[] memory _asset_quantities_long) external;
    }
}