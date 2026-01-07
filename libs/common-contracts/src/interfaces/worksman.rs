use alloy_sol_types::sol;

sol! {
    interface IWorksman  {
        function addVault(address vault) external;

        function buildVault(uint128 index, bytes calldata info) external returns (address);
    }
}