use alloy_sol_types::sol;

sol! {
    interface IGuildmaster  {
        function submitIndex(uint128 index, string calldata name, string calldata symbol, string calldata description, string calldata methodology, uint128 initial_price, address curator, string calldata custody) external;

        function beginEditIndex(uint128 index) external;

        function finishEditIndex(uint128 index) external;

        function submitAssetWeights(uint128 index, bytes calldata asset_names, bytes calldata asset_weights) external;

        function submitVote(uint128 index, bytes calldata vote) external;

        function updateIndexQuote(uint128 vendor_id, uint128 index_id) external;

        function updateMultipleIndexQuotes(uint128 vendor_id, uint128[] memory index_ids) external;

        event BeginEditIndex(uint128 index, address sender);

        event FinishEditIndex(uint128 index, address sender);

        event IndexCreated(uint128 index, string name, string symbol, address vault);
    }
}