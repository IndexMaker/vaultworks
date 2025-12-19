use abacus_macros::abacus;

/// Create market adding initial assets and setting market vectors
pub fn create_market(
    asset_names_id: u128,
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

        LDL         asset_names_id                  // Stack [AN = AssetNames]
        ZEROS       0                               // Stack [AN, Z]
        LDD         1                               // Stack [AN, Z, AN]
        STR         _AssetNames                     // Stack [AN, Z]
        LDD         0                               // Stack [AN, Z, Z]
        STR         _AssetPrices                    // Stack [AN, Z]
        LDD         0                               // Stack [AN, Z, Z]
        STR         _AssetSlopes                    // Stack [AN, Z]
        LDD         0                               // Stack [AN, Z, Z]
        STR         _AssetLiquidity                 // Stack [AN, Z]
        LDD         0                               // Stack [AN, Z, Z]
        STR         _SupplyShort                    // Stack [AN, Z]
        LDD         0                               // Stack [AN, Z, Z]
        STR         _SupplyLong                     // Stack [AN, Z]
        LDD         0                               // Stack [AN, Z, Z]
        STR         _DemandShort                    // Stack [AN, Z]
        LDD         0                               // Stack [AN, Z, Z]
        STR         _DemandLong                     // Stack [AN, Z]
        LDD         0                               // Stack [AN, Z, Z]
        STR         _DeltaShort                     // Stack [AN, Z]
        LDD         0                               // Stack [AN, Z, Z]
        STR         _DeltaLong                      // Stack [AN, Z]
        LDD         0                               // Stack [AN, Z, Z]
        STR         _Margin                         // Stack [AN, Z]

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