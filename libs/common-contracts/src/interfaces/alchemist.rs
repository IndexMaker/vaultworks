use alloy_sol_types::sol;

sol! {
    interface IAlchemist  {
    function submitAssetWeights(uint128 index_id, bytes calldata asset_names, bytes calldata asset_weights) external;

    function processPendingRebalance(uint128 vendor_id, uint128 index_id, uint128 capacity_factor) external returns (bytes[] memory);
}
}