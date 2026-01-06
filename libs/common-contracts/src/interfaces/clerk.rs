use alloy_sol_types::sol;

sol! {
    interface IClerk  {
        function initialize(address owner, address abacus) external;

        function store(uint128 id, bytes calldata data) external;

        function load(uint128 id) external view returns (bytes memory);
    }
}