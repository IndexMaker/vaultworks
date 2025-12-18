use alloy_sol_types::sol;

sol! {
    interface IClerk {
        function execute(uint8[] memory code, uint128 num_registry) external;
    }
}