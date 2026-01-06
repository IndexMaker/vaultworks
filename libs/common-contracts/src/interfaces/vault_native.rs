use alloy_sol_types::sol;

sol!{
    interface IVaultNative  {
        function configureRequests(uint128 vendor_id, address custody, address asset, uint128 max_order_size) external;

        function setOperator(address operator, bool status) external;

        function isOperator(address operator) external view returns (bool);

        function collateralAsset() external view returns (address);

        function assetsValue(address account) external view returns (uint128);

        function totalAssetsValue() external view returns (uint128);

        function convertAssetsValue(uint128 shares) external view returns (uint128);

        function convertItpAmount(uint128 assets) external view returns (uint128);

        function estimateAcquisitionCost(uint128 shares) external view returns (uint128);

        function estimateAcquisitionItp(uint128 assets) external view returns (uint128);

        function estimateDisposalGains(uint128 shares) external view returns (uint128);

        function estimateDisposalItpCost(uint128 assets) external view returns (uint128);

        function placeBuyOrder(uint128 collateral_amount, bool instant_fill, address trader) external;

        function placeSellOrder(uint128 itp_amount, bool instant_fill, address trader) external;

        function confirmBuyOrder(uint128 amount, address trader) external;

        function confirmSellOrder(uint128 itp_amount, address trader) external;

        function confirmWithdrawAvailable(uint128 itp_amount, address trader) external;

        function withdrawGains(uint128 amount, address trader) external;

        function getPendingAcquisitionCollateral(address trader) external view returns (uint128);

        function getPendingDisposalItp(address trader) external view returns (uint128);

        function getWithdrawAvailable(address trader) external view returns (uint128);

        function getActiveAcquisitionCollateral(address trader) external view returns (uint128);

        function getActiveDisposalItp(address trader) external view returns (uint128);
    }
}