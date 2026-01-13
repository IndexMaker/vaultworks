use alloy_sol_types::sol;

sol!{
    interface IVaultNativeClaims  {
        function getPendingOrder(address keeper, address trader) external view returns (uint128, uint128);

        function getClaimableAcquisition(address keeper) external view returns (uint128, uint128);

        function getClaimableDisposal(address keeper) external view returns (uint128, uint128);

        function claimAcquisition(uint128 collateral_amount, address keeper, address trader) external returns (uint128);

        function claimDisposal(uint128 itp_amount, address keeper, address trader) external returns (uint128);
            
        event AcquisitionClaim(address controller, uint128 index_id, uint128 vendor_id, uint128 remain, uint128 spent, uint128 itp_minted);

        event DisposalClaim(address controller, uint128 index_id, uint128 vendor_id, uint128 itp_remain, uint128 itp_burned, uint128 gains);
    }
}