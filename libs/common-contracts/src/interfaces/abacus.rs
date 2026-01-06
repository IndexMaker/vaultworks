use alloy_sol_types::sol;

sol! {
    interface IAbacus  {
        function execute(bytes calldata code, uint128 num_registry) external;
    }
}