use alloy_sol_types::sol;

sol! {
    interface IWorksman  {
        function addVault(address vault) external;

        function buildVault() external returns (address);
    }
}