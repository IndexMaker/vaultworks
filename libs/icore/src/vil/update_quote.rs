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
        LDV         index_asset_names_id            //  [AssetNames]
        LDV         market_asset_names_id           //  [AssetNames, MarketAssetNames]
        
        // Compute P = MarketAssetPrices * AssetWeights
        LDV         asset_prices_id                 //  [AssetNames, MarketAssetNames, MarketAssetPrices]
        JFLT        1   2                           //  [AssetNames, MarketAssetNames, Flt_MarketAssetPrices]
        LDR         _AssetWeights                   //  [AssetNames, MarketAssetNames, Flt_MarketAssetPrices, AssetWeights]
        SWAP        1                               //  [AssetNames, MarketAssetNames, AssetWeights, Flt_MarketAssetPrices]
        MUL         1                               //  [AssetNames, MarketAssetNames, AssetWeights, P = (AssetWeights * Flt_MarketAssetPrices)]
        STR         _Price                          //  [AssetNames, MarketAssetNames, AssetWeights]
        
        // Compute S = MarketAssetSlopes * AssetWeights^2
        MUL         0                               //  [AssetNames, MarketAssetNames, AssetWeights^2]
        LDV         asset_slopes_id                 //  [AssetNames, MarketAssetNames, AssetWeights^2, MarketAssetSlopes]
        JFLT        2   3                           //  [AssetNames, MarketAssetNames, AssetWeights^2, Flt_MarketAssetSlopes]
        MUL         1                               //  [AssetNames, MarketAssetNames, AssetWeights^2, S = (Flt_MarketAssetSlopes * AssetWeights^2)]
        STR         _Slope                          //  [AssetNames, MarketAssetNames, AssetWeights^2]
        POPN        1                               //  [AssetNames, MarketAssetNames]

        // Compute C = MIN(MarketAssetLiquidity / AssetWeights) 
        //
        LDR         _AssetWeights                   //  [AssetNames, MarketAssetNames, AssetWeights]
        LDV         asset_liquidity_id              //  [AssetNames, MarketAssetNames, AssetWeights, MarketAssetLiquidity]
        JFLT        2   3                           //  [AssetNames, MarketAssetNames, AssetWeights, Flt_MarketAssetLiquidity]
        DIV         1                               //  [AssetNames, MarketAssetNames, AssetWeights, C_vec = (Flt_MarketAssetLiquidity / AssetWeights)]
        VMIN                                        //  [AssetNames, MarketAssetNames, AssetWeights, C = MIN(C_vec)]
        STR         _Capacity                       //  [AssetNames, MarketAssetNames, AssetWeights]
        POPN        3                               //  []

        // =============================
        // * * * COMMIT NEW VALUES * * *
        // =============================

        LDM         _Capacity                       //  [Capacity]
        LDM         _Price                          //  [Capacity, Price
        LDM         _Slope                          //  [Capacity, Price, Slope]
        PKV         3                               //  [(Capacity, Price, Slope)]
        STV         quote_id
    }
}
