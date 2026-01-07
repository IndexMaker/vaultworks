use alloy_sol_types::sol;

sol! {
    interface IVaultRequests  {
        function configureRequests(uint128 vendor_id, address custody, address asset) external;

        function asset() external view returns (address);

        function assets(address account) external view returns (uint256);

        function totalAssets() external view returns (uint256);

        function convertToAssets(uint256 shares) external view returns (uint256);

        function convertToShares(uint256 assets) external view returns (uint256);

        function isOperator(address owner, address operator) external view returns (bool);

        function setOperator(address operator, bool approved) external returns (bool);

        function requestDeposit(uint256 assets, address controller, address owner) external returns (uint256);

        function pendingDepositRequest(uint256 request_id, address controller) external view returns (uint256);

        function claimableDepositRequest(uint256 request_id, address controller) external view returns (uint256);

        function claimableDepositUpdate(uint256 request_id, uint256 assets, uint256 shares) external view;

        function deposit(uint256 assets, address receiver, address controller) external returns (uint256);

        function requestRedeem(uint256 shares, address controller, address owner) external returns (uint256);

        function pendingRedeemRequest(uint256 request_id, address controller) external view returns (uint256);

        function claimableRedeemRequest(uint256 request_id, address controller) external view returns (uint256);

        function claimableRedeemUpdate(uint256 request_id, uint256 shares, uint256 assets) external view;

        function redeem(uint256 shares, address receiver, address controller) external returns (uint256);
    }
}