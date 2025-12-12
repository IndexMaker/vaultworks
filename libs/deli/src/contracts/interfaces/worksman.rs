use alloy_sol_types::sol;

sol! {
    interface IWorksman  {
        function acceptAppointment(address castle) external;

        function buildVault(uint128 index, uint8[] memory info) external returns (address);

        function addVault(address vault) external;
    }
}