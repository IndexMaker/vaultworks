use abacus_macros::abacus;

/// Add assets and extend all market vectors
/// 
/// We only support adding new assets to market vectors.
/// 
/// Note: Any potential implementation of the removal
/// of assets from market vectors would need to check
/// whether: |Supply| + |Demand| == 0.
/// 
pub fn add_market_assets(
    asset_names_added_id: u128,
    market_asset_names_id: u128,
    market_asset_prices_id: u128,
    market_asset_slopes_id: u128,
    market_asset_liquidity_id: u128,
    supply_long_id: u128,
    supply_short_id: u128,
    demand_long_id: u128,
    demand_short_id: u128,
    delta_long_id: u128,
    delta_short_id: u128,
    margin_id: u128,
) -> Vec<u8> {
    abacus! {
        // ====================================
        // * * * (TRY) COMPUTE NEW VALUES * * *
        // ====================================

        LDL         market_asset_names_id           // Stack [AN_old = MarketAssetNames]
        LDL         asset_names_added_id            // Stack [AN_old, AN_extra = AssetNamesAdded]
        LUNION      1                               // Stack [AN_old, AN_new = AN_old U AN_extra]
        SWAP        1                               // Stack [AN_new, AN_old]

        LDD         1                               // Stack [AN_new, AN_old, AN_new]
        STR         _AssetNames                     // Stack [AN_new, AN_old]

        LDV         market_asset_prices_id          // Stack [AN_new, AN_old, AP_old]
        ZEROS       2                               // Stack [AN_new, AN_old, AP_old, (0..)]
        JUPD        1   3   2                       // Stack [AN_new, AN_old, AP_old, AP_new]
        STR         _AssetPrices                    // Stack [AN_new, AN_old, AP_old]
        POPN        1                               // Stack [AN_new, AN_old]

        LDV         market_asset_slopes_id          // Stack [AN_new, AN_old, AS_old]
        ZEROS       2                               // Stack [AN_new, AN_old, AS_old, (0..)]
        JUPD        1   3   2                       // Stack [AN_new, AN_old, AS_old, AS_new]
        STR         _AssetSlopes                    // Stack [AN_new, AN_old, AS_old]
        POPN        1                               // Stack [AN_new, AN_old]

        LDV         market_asset_liquidity_id       // Stack [AN_new, AN_old, AL_old]
        ZEROS       2                               // Stack [AN_new, AN_old, AL_old, (0..)]
        JUPD        1   3   2                       // Stack [AN_new, AN_old, AL_old, AL_new]
        STR         _AssetLiquidity                 // Stack [AN_new, AN_old, AL_old]
        POPN        1                               // Stack [AN_new, AN_old]

        LDV         supply_short_id                 // Stack [AN_new, AN_old, SS_old]
        ZEROS       2                               // Stack [AN_new, AN_old, SS_old, (0..)]
        JUPD        1   3   2                       // Stack [AN_new, AN_old, SS_old, SS_new]
        STR         _SupplyShort                    // Stack [AN_new, AN_old, SS_old]
        POPN        1                               // Stack [AN_new, AN_old]

        LDV         supply_long_id                  // Stack [AN_new, AN_old, SL_old]
        ZEROS       2                               // Stack [AN_new, AN_old, SL_old, (0..)]
        JUPD        1   3   2                       // Stack [AN_new, AN_old, SL_old, SL_new]
        STR         _SupplyLong                     // Stack [AN_new, AN_old, SL_old]
        POPN        1                               // Stack [AN_new, AN_old]

        LDV         demand_short_id                 // Stack [AN_new, AN_old, DS_old]
        ZEROS       2                               // Stack [AN_new, AN_old, DS_old, (0..)]
        JUPD        1   3   2                       // Stack [AN_new, AN_old, DS_old, DS_new]
        STR         _DemandShort                    // Stack [AN_new, AN_old, DS_old]
        POPN        1                               // Stack [AN_new, AN_old]

        LDV         demand_long_id                  // Stack [AN_new, AN_old, DL_old]
        ZEROS       2                               // Stack [AN_new, AN_old, DL_old, (0..)]
        JUPD        1   3   2                       // Stack [AN_new, AN_old, DL_old, DL_new]
        STR         _DemandLong                     // Stack [AN_new, AN_old, DL_old]
        POPN        1                               // Stack [AN_new, AN_old]

        LDV         delta_short_id                  // Stack [AN_new, AN_old, DS_old]
        ZEROS       2                               // Stack [AN_new, AN_old, DS_old, (0..)]
        JUPD        1   3   2                       // Stack [AN_new, AN_old, DS_old, DS_new]
        STR         _DeltaShort                     // Stack [AN_new, AN_old, DS_old]
        POPN        1                               // Stack [AN_new, AN_old]

        LDV         delta_long_id                   // Stack [AN_new, AN_old, DL_old]
        ZEROS       2                               // Stack [AN_new, AN_old, DL_old, (0..)]
        JUPD        1   3   2                       // Stack [AN_new, AN_old, DL_old, DL_new]
        STR         _DeltaLong                      // Stack [AN_new, AN_old, DL_old]
        POPN        1                               // Stack [AN_new, AN_old]
        
        LDV         margin_id                       // Stack [AN_new, AN_old, M_old]
        ZEROS       2                               // Stack [AN_new, AN_old, M_old, (0..)]
        JUPD        1   3   2                       // Stack [AN_new, AN_old, M_old, M_new]
        STR         _Margin                         // Stack [AN_new, AN_old, M_old]
        POPN        1                               // Stack [AN_new, AN_old]

        POPN        2                               // Stack []
        
        // =============================
        // * * * COMMIT NEW VALUES * * *
        // =============================

        LDM         _AssetNames
        STL         market_asset_names_id

        LDM         _AssetPrices
        LDM         _AssetSlopes
        LDM         _AssetLiquidity
        STV         market_asset_liquidity_id
        STV         market_asset_slopes_id
        STV         market_asset_prices_id

        LDM         _SupplyLong
        LDM         _SupplyShort
        STV         supply_short_id
        STV         supply_long_id

        LDM         _DemandLong
        LDM         _DemandShort
        STV         demand_short_id
        STV         demand_long_id

        LDM         _DeltaLong
        LDM         _DeltaShort
        STV         delta_short_id
        STV         delta_long_id

        LDM         _Margin
        STV         margin_id
    }
}