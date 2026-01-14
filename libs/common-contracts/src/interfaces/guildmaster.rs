use alloy_sol_types::sol;

sol! {
    interface IGuildmaster  {
        function submitIndex(uint128 vendor_id, uint128 index_id, string calldata name, string calldata symbol, string calldata description, string calldata methodology, uint128 initial_price, address curator, string calldata custody, address[] memory operators, address collateral_custody, address collateral_asset, uint128 max_order_size) external returns (address);

        function beginEditIndex(uint128 index_id) external;

        function finishEditIndex(uint128 index_id) external;

        function submitAssetWeights(uint128 index_id, bytes calldata asset_names, bytes calldata asset_weights) external;

        function submitVote(uint128 index_id, bytes calldata vote) external;

        event BeginEditIndex(uint128 index_id, address sender);

        event FinishEditIndex(uint128 index_id, address sender);

        event IndexCreated(uint128 index_id, string name, string symbol, address vault);
        
        event IndexWeightsUpdated(uint128 index_id, address sender);
        
        event IndexVoteUpdated(uint128 index_id, address sender);
    }
}