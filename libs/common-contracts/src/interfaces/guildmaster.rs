use alloy_sol_types::sol;

sol! {
    interface IGuildmaster  {
        function submitIndex(uint128 index, bytes calldata asset_names, bytes calldata asset_weights, bytes calldata info) external;

        function submitVote(uint128 index, bytes calldata vote) external;

        function updateIndexQuote(uint128 vendor_id, uint128 index_id) external;

        function updateMultipleIndexQuotes(uint128 vendor_id, uint128[] memory index_ids) external;
    }
}