use abacus_macros::abacus;

/// Update Market Data
///
pub fn update_market_data(
    asset_names_id: u128,
    asset_prices_id: u128,
    asset_slopes_id: u128,
    asset_liquidity_id: u128,
    market_asset_names_id: u128,
    market_asset_prices_id: u128,
    market_asset_slopes_id: u128,
    market_asset_liquidity_id: u128,
) -> Result<Vec<u8>, Vec<u8>> {
    abacus! {
        // ====================================
        // * * * (TRY) COMPUTE NEW VALUES * * *
        // ====================================

        // Update Prices & Slopes & Liquidity
        LDL         asset_names_id              // Stack [AN = AssetNames]
        LDL         market_asset_names_id       // Stack [AN = AssetNames, MAN = MarketAssetNames]

        // Compute MarketAssetPrices j= AssetPrices
        LDV         asset_prices_id             // Stack [AN, MAN, AP]
        LDV         market_asset_prices_id      // Stack [AN, MAN, AP, MAP]
        JUPD        1   2   3                   // Stack [AN, MAN, AP, MAP_updated = (MAP j= AP)]
        STR         _Prices                     // Stack [AN, MAN, AP]
        POPN        1                           // Stack [AN, MAN]

        // Compute MarketAssetSlopes j= AssetSlopes
        LDV         asset_slopes_id             // Stack [AN, MAN, AS]
        LDV         market_asset_slopes_id      // Stack [AN, MAN, AS, MAS]
        JUPD        1   2   3                   // Stack [AN, MAN, AS, MAS_updated = (MAS j= AS)]
        STR         _Slopes                     // Stack [AN, MAN, AS]
        POPN        1                           // Stack [AN, MAN]

        // Compute MarketAssetLiquidity j= AssetLiquidity
        LDV         asset_liquidity_id          // Stack [AN, MAN, AL]
        LDV         market_asset_liquidity_id   // Stack [AN, MAN, AL, MAL]
        JUPD        1   2   3                   // Stack [AN, MAN, AL, MAL_updated = (MAL J= AL)]
        STR         _Liquidity                  // Stack [AN, MAN, AL]
        POPN        1                           // Stack [AN, MAN]

        // =============================
        // * * * COMMIT NEW VALUES * * *
        // =============================

        LDM         _Prices
        LDM         _Slopes
        LDM         _Liquidity
        STV         market_asset_liquidity_id
        STV         market_asset_slopes_id
        STV         market_asset_prices_id
    }
}
