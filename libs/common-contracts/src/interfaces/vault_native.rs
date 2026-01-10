use alloy_sol_types::sol;

sol! {
    interface IVaultNative  {
        function configureRequests(uint128 vendor_id, address custody, address asset, uint128 max_order_size) external;

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

        function placeBuyOrder(uint128 collateral_amount, bool instant_fill, address operator) external returns (uint128, uint128, uint128);

        function placeSellOrder(uint128 itp_amount, bool instant_fill, address operator) external returns (uint128, uint128, uint128);

        function processPendingBuyOrder() external returns (uint128, uint128, uint128);

        function processPendingSellOrder() external returns (uint128, uint128, uint128);

        function getClaimableAcquisitionCost(address operator) external view returns (uint128);

        function getClaimableDisposalItpCost(address operator) external view returns (uint128);

        function claimAcquisitionCost(address trader, uint128 amount) external returns (uint128);

        function claimDisposalItpCost(address trader, uint128 itp_amount) external;

        event OperatorSet(address controller, address operator, bool approved);
    }
}
