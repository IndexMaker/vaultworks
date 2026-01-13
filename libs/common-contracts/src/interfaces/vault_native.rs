use alloy_sol_types::sol;

sol! {
    interface IVaultNative  {
        function configureRequests(uint128 vendor_id, address custody, address asset, uint128 max_order_size) external;

        function isOperator(address owner, address operator) external view returns (bool);

        function setOperator(address operator, bool approved) external returns (bool);

        function setAdminOperator(address controller, bool approved) external;

        function collateralAsset() external view returns (address);

        function vendorId() external view returns (uint128);

        function custodyAddress() external view returns (address);

        function assetsValue(address account) external view returns (uint128);

        function totalAssetsValue() external view returns (uint128);

        function convertAssetsValue(uint128 shares) external view returns (uint128);

        function convertItpAmount(uint128 assets) external view returns (uint128);

        function estimateAcquisitionCost(uint128 shares) external view returns (uint128);

        function estimateAcquisitionItp(uint128 assets) external view returns (uint128);

        function estimateDisposalGains(uint128 shares) external view returns (uint128);

        function estimateDisposalItpCost(uint128 assets) external view returns (uint128);

        function getMaxOrderSize() external view returns (uint128);

        function getQuote() external view returns (uint128, uint128, uint128);

        function syncTotalSupply() external returns (uint256);

        function syncBalanceOf(address account) external returns (uint256);

        event OperatorSet(address controller, address operator, bool approved);
    }
}
