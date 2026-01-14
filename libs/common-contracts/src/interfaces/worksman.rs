use alloy_sol_types::sol;

sol! {
    interface IWorksman  {
        function setVaultPrototype(address vault_implementation) external;

        function buildVault() external returns (address);
    }
}