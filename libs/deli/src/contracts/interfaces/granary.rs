use alloy_sol_types::sol;

sol! {
    interface IGranary {
        function store(uint128 id, uint8[] memory data) external;

        function fetch(uint128 id) external view returns (uint8[] memory);
    }
}