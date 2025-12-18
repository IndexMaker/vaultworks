use alloy_sol_types::sol;

sol!{
    interface ITreasury  {
        function mint(address to, uint256 value) external;

        function name() external view returns (string memory);

        function symbol() external view returns (string memory);

        function decimals() external view returns (uint8);

        function websiteUrl() external view returns (string memory);

        function balanceOf(address account) external view returns (uint256);

        function totalSupply() external view returns (uint256);

        function transfer(address to, uint256 value) external returns (bool);

        function transferFrom(address from, address to, uint256 value) external returns (bool);

        function allowance(address owner, address spender) external view returns (uint256);

        function approve(address spender, uint256 value) external returns (bool);

        function upgradeToAndCall(address new_implementation, bytes calldata data) external;

        function owner() external view returns (address);

        function transferOwnership(address new_owner) external;

        function renounceOwnership() external;

        function initialize(address owner) external;

        function setVersion() external;

        function getVersion() external view returns (uint32);

        error OwnableUnauthorizedAccount(address);

        error OwnableInvalidOwner(address);

        error UUPSUnauthorizedCallContext();

        error UUPSUnsupportedProxiableUUID(bytes32);

        error ERC1967InvalidImplementation(address);

        error ERC1967InvalidAdmin(address);

        error ERC1967InvalidBeacon(address);

        error ERC1967NonPayable();

        error AddressEmptyCode(address);

        error FailedCall();

        error FailedCallWithReason(bytes);

        error InvalidInitialization();

        error InvalidVersion(uint32);

        error ERC20InsufficientBalance(address, uint256, uint256);

        error ERC20InvalidSender(address);

        error ERC20InvalidReceiver(address);

        error ERC20InsufficientAllowance(address, uint256, uint256);

        error ERC20InvalidSpender(address);

        error ERC20InvalidApprover(address);
    }
}