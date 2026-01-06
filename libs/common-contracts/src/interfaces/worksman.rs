use alloy_sol_types::sol;

sol! {
    interface IWorksman  {
        function acceptAppointment(address worksman) external;

        function buildVault(uint128 index, bytes calldata info) external returns (address);

        function addVault(address vault) external;
    }
}