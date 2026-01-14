use abacus_macros::abacus;

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
) -> Result<Vec<u8>, Vec<u8>> {
    abacus! {
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

        // Compute C = MIN(AssetLiquidity / AssetWeights)
        //
        // NOTE: We just put market liquidity based capacity, and then when we execute orders we cap with available margin.
        //
        LDR         _AssetWeights                   //  [AN = AssetNames, MAN = MarketAssetNames, W = AssetWeights]
        LDV         asset_liquidity_id              //  [AN, MAN, W, MAL = MarketAssetLiquidity]
        JFLT        2   3                           //  [AN, MAN, W, Flt_MAL]
        DIV         1                               //  [AN, MAN, W, C_vec = (Flt_MAL / W)]
        VMIN                                        //  [AN, MAN, W, C = MIN(C_vec)]
        STR         _Capacity                       //  [AN, MAN, W]
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
