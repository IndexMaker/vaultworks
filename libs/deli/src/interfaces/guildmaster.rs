use alloy_sol_types::sol;

sol! {
    interface IGuildmaster  {
        function submitIndex(
            uint128 index, 
            uint8[] memory asset_names, 
            uint8[] memory asset_weights, 
            uint8[] memory info) external;

        function submitVote(
            uint128 index, 
            uint8[] memory vote) external;
    }
}