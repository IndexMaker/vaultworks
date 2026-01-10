use alloy_sol_types::sol;

sol! {
    interface IVaultNativeOrders  {
        function placeBuyOrder(uint128 collateral_amount, bool instant_fill, address keeper, address trader) external returns (uint128, uint128, uint128);

        function placeSellOrder(uint128 itp_amount, bool instant_fill, address keeper, address trader) external returns (uint128, uint128, uint128);

        function processPendingBuyOrder(address keeper) external returns (uint128, uint128, uint128);

        function processPendingSellOrder(address keeper) external returns (uint128, uint128, uint128);
        
        event BuyOrder(address keeper, address trader, uint128 index_id, uint128 vendor_id, uint128 collateral_amount);

        event SellOrder(address keeper, address trader, uint128 index_id, uint128 vendor_id, uint128 itp_amount);

        event Acquisition(address controller, uint128 index_id, uint128 vendor_id, uint128 remain, uint128 spent, uint128 itp_minted);

        event Disposal(address controller, uint128 index_id, uint128 vendor_id, uint128 itp_remain, uint128 itp_burned, uint128 gains);
    }
}