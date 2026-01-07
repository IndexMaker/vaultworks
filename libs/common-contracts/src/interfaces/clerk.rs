use alloy_sol_types::sol;

sol! {
    interface IClerk  {
        function fetchVector(uint128 id) external view returns (bytes memory);

        function updateRecords(bytes calldata code, uint128 num_registry) external;
    }
}