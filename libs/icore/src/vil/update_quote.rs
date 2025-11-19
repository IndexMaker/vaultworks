use devil_macros::devil;

/// Update Index Quote (Capacity, Price, Slope)
/// 
pub fn update_quote(
    index_asset_names_id: u128,
    weights_id: u128,
    quote_id: u128,
    market_asset_names_id: u128,
    asset_prices_id: u128,
    asset_slopes_id: u128,
    asset_liquidity_id: u128,
    delta_long_id: u128,
    delta_short_id: u128,
    margin_id: u128,
) -> Vec<u128> {
    devil! {
        // ====================================
        // * * * (TRY) COMPUTE NEW VALUES * * *
        // ====================================

        LDV         weights_id                      //  [AssetWeights]
        STR         _AssetWeights

        // Load AssetNames & MarketAssetNames
        LDL         index_asset_names_id            //  [AssetNames]
        LDL         market_asset_names_id           //  [AssetNames, MarketAssetNames]
        
        // Compute P = MarketAssetPrices * AssetWeights
        LDV         asset_prices_id                 //  [AssetNames, MarketAssetNames, MarketAssetPrices]
        JFLT        1   2                           //  [AssetNames, MarketAssetNames, Flt_MarketAssetPrices]
        LDR         _AssetWeights                   //  [AssetNames, MarketAssetNames, Flt_MarketAssetPrices, AssetWeights]
        SWAP        1                               //  [AssetNames, MarketAssetNames, AssetWeights, Flt_MarketAssetPrices]
        MUL         1                               //  [AssetNames, MarketAssetNames, AssetWeights, P_vec = (AssetWeights * Flt_MarketAssetPrices)]
        VSUM                                        //  [AssetNames, MarketAssetNames, AssetWeights, P = SUM(P_vec[..])]
        STR         _Price                          //  [AssetNames, MarketAssetNames, AssetWeights]
        
        // Compute S = MarketAssetSlopes * AssetWeights^2
        MUL         0                               //  [AssetNames, MarketAssetNames, AssetWeights^2]
        LDV         asset_slopes_id                 //  [AssetNames, MarketAssetNames, AssetWeights^2, MarketAssetSlopes]
        JFLT        2   3                           //  [AssetNames, MarketAssetNames, AssetWeights^2, Flt_MarketAssetSlopes]
        MUL         1                               //  [AssetNames, MarketAssetNames, AssetWeights^2, S_vec = (Flt_MarketAssetSlopes * AssetWeights^2)]
        VSUM                                        //  [AssetNames, MarketAssetNames, AssetWeights^2, S = SUM(S_vec[..])]
        STR         _Slope                          //  [AssetNames, MarketAssetNames, AssetWeights^2]
        POPN        1                               //  [AssetNames, MarketAssetNames]

        // Compute C = MIN(AssetCapacity / AssetWeights) 
        // &
        // AssetCapacity = MIN(MarketAssetLiquidity, Margin - MAX(Delta Short, Delta Long))
        //
        // Note that this will guarantee that order is capped to either market liquidity or remaining margin
        //
        // Possible Changes
        // ----------------
        // 1. Add InventoryCapacity
        //
        //      AvailableMargin = Margin - MAX(DeltaLong, DeltaShort)
        //	
        //      MarketCapacity = MIN(Liquidity, AvailableMargin)
        // 
        //      InventoryCapacity = MIN(SupplyLong, DeltaLong)
        // 
        //      Capacity = MAX(MarketCapacity, InventoryCapacity)
        //
        //  2. Consider whether we need DemandShort and SupplyShort
        //
        //      DeltaLong - DeltaShort = SupplyLong - DemandLong

        //  for:
        //      DemandShort = (0,...)
        //      SupplyShort = (0,...)
        //  
        //  We would always have Demand and Supply Long never Short, only Delta can be Long and Short.
        //  Demand could only be short if we were to allow short-selling Index, and then Supply could
        //  be Short if we are to allow Vendor to go short. We keep Short side for Supply and Demand
        //  for accounting correctness, but it should always be (0,...), and it might be more gas
        //  efficient to skip that and only have Long sides for Supply and Demand.
        //
        //  TVL vs Supply & Demand
        //  ----------------------
        //  Both Supply & Demand constitute TVL tracking mechanism, however TVL can be seen as static
        //  whereas Supply & Demand are dynamic. When Delta = (0,...), then TVL = Supply = Demand, and
        //  when Supply != Demand, then TVL is blurred, because it means that both Vendor and users
        //  are acting at the same time, and while Vendor tries to reduce Delta to (0,...), and users
        //  move Delta away from (0,...).
        //
        LDV         delta_short_id                  //  [AssetNames, MarketAssetNames, DS = DeltaShort]
        LDV         delta_long_id                   //  [AssetNames, MarketAssetNames, DS, DL = DeltaLong]
        MAX         1                               //  [AssetNames, MarketAssetNames, DS, Max_D = MAX(DS, DL)]
        LDV         margin_id                       //  [AssetNames, MarketAssetNames, DS, Max_D = MAX(DS, DL), Margin]
        SSB         1                               //  [AssetNames, MarketAssetNames, DS, Max_D, RM = Margin s- Max_D]
        SWAP        2                               //  [AssetNames, MarketAssetNames, RM, Max_D, DS]
        POPN        2                               //  [AssetNames, MarketAssetNames, RM]
        LDV         asset_liquidity_id              //  [AssetNames, MarketAssetNames, RM, MarketAssetLiquidity]
        MIN         1                               //  [AssetNames, MarketAssetNames, RM, AC = MIN(MarketAssetLiquidity, RM)]
        LDR         _AssetWeights                   //  [AssetNames, MarketAssetNames, RM, AC, AssetWeights]
        SWAP        2                               //  [AssetNames, MarketAssetNames, AssetWeights, AC, RM]
        POPN        1                               //  [AssetNames, MarketAssetNames, AssetWeights, AC]
        JFLT        2   3                           //  [AssetNames, MarketAssetNames, AssetWeights, Flt_AC]
        DIV         1                               //  [AssetNames, MarketAssetNames, AssetWeights, C_vec = (Flt_AC / AssetWeights)]
        VMIN                                        //  [AssetNames, MarketAssetNames, AssetWeights, C = MIN(C_vec)]
        STR         _Capacity                       //  [AssetNames, MarketAssetNames, AssetWeights]
        POPN        3                               //  []

        // =============================
        // * * * COMMIT NEW VALUES * * *
        // =============================

        LDM         _Capacity                       //  [Capacity]
        LDM         _Price                          //  [Capacity, Price]
        LDM         _Slope                          //  [Capacity, Price, Slope]
        PKV         3                               //  [(Capacity, Price, Slope)]
        STV         quote_id
    }
}
