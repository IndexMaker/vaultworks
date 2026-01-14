use alloy_sol_types::sol;

sol!{
    interface IVault  {
        function initialize(address owner, address vault_implementation, address gate_to_castle) external;

        function installOrders(address orders_implementation) external;

        function installClaims(address claims_implementation) external;

        function cloneImplementation(address to, address new_owner) external;

        function castle() external view returns (address);

        function vaultImplementation() external view returns (address);

        function ordersImplementation() external view returns (address);

        function claimsImplementation() external view returns (address);

        function setVersion() external;

        function getVersion() external view returns (uint32);

        function UPGRADE_INTERFACE_VERSION() external view returns (string memory);

        function upgradeToAndCall(address new_implementation, bytes calldata data) external payable;

        function proxiableUuid() external view returns (bytes32);

        function owner() external view returns (address);

        function transferOwnership(address new_owner) external;

        function renounceOwnership() external;

        function configureVault(uint128 index_id, string calldata name, string calldata symbol, string calldata description, string calldata methodology, uint128 initial_price, address curator, string calldata custody) external;

        function indexId() external view returns (uint128);

        function description() external view returns (string memory);

        function methodology() external view returns (string memory);

        function initialPrice() external view returns (uint128);

        function curator() external view returns (address);

        function custody() external view returns (string memory);

        function name() external view returns (string memory);

        function symbol() external view returns (string memory);

        function decimals() external view returns (uint8);

        function totalSupply() external view returns (uint256);

        function balanceOf(address account) external view returns (uint256);

        function transfer(address to, uint256 value) external returns (bool);

        function allowance(address owner, address spender) external view returns (uint256);

        function approve(address spender, uint256 value) external returns (bool);

        function transferFrom(address from, address to, uint256 value) external returns (bool);

        function addCustodian(address account) external;

        function addCustodians(address[] memory accounts) external;

        function removeCustodian(address account) external;

        function isCustodian(address account) external view returns (bool);
        
        event CustodianSet(address account, bool is_custodian);
    }
}