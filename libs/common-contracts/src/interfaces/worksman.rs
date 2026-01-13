use alloy_sol_types::sol;

sol! {
    interface IWorksman  {
        function addVault(address vault) external;

        function buildVault(uint128 index, string calldata name, string calldata symbol, string calldata description, string calldata methodology, uint128 initial_price, address curator, string calldata custody) external returns (address);

        event VautlDeployed(uint128 index, string name, string symbol, address vault);
    }
}