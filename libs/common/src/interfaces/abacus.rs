use alloy_sol_types::sol;

sol! {
    interface IAbacus {
        function execute(uint8[] memory code, uint128 num_registry) external;
    }
}