use alloy_sol_types::sol;

sol! {
    interface IClerk  {
        function updateRecords(bytes calldata code, uint128 num_registry) external;
    }
}