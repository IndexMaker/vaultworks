use alloy_sol_types::sol;

sol!{
    interface IVault  {
        function initialize(address owner, address requests, address gate_to_castle) external;

        function setVersion() external;

        function getVersion() external view returns (uint32);

        function owner() external view returns (address);

        function transferOwnership(address new_owner) external;

        function renounceOwnership() external;

        function configureVault(uint128 index_id, string calldata name, string calldata symbol) external;

        function name() external view returns (string memory);

        function symbol() external view returns (string memory);

        function decimals() external view returns (uint8);

        function totalSupply() external view returns (uint256);

        function balanceOf(address account) external view returns (uint256);

        function transfer(address to, uint256 value) external;

        function allowance(address owner, address spender) external view returns (uint256);

        function approve(address spender, uint256 value) external returns (bool);

        function transferFrom(address from, address to, uint256 value) external returns (bool);

        function UPGRADE_INTERFACE_VERSION() external view returns (string memory);

        function upgradeToAndCall(address new_implementation, bytes calldata data) external payable;

        function proxiableUuid() external view returns (bytes32);
    }
}