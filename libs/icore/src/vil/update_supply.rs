use devil_macros::devil;

/// Update Market (Supply, Delta)
/// 
pub fn update_supply(
    asset_names_id: u128,
    asset_quantities_short_id: u128,
    asset_quantities_long_id: u128,
    market_asset_names_id: u128,
    supply_long_id: u128,
    supply_short_id: u128,
    demand_long_id: u128,
    demand_short_id: u128,
    delta_long_id: u128,
    delta_short_id: u128,
) -> Vec<u128> {
    devil! {
        // ====================================
        // * * * (TRY) COMPUTE NEW VALUES * * *
        // ====================================

        // Match Market: Update Demand and Delta
        LDL         asset_names_id              // Stack [MAN = AssetNames]
        LDL         market_asset_names_id       // Stack [AN = AssetNames, MAN = MarketAssetNames]

        // Compute SupplyShort <- AssetQuantitiesShort
        LDV         asset_quantities_short_id   // Stack [AN, MAN, AQS]
        LDV         supply_short_id             // Stack [AN, MAN, AQS, SS = SupplyShort]
        JUPD        1   2   3                   // Stack [AN, MAN, AQS, SS_updated]
        STR         _SupplyShort                // Stack [AN, MAN, AQS]
        POPN        1                           // Stack [AN, MAN]

        // Compute SupplyLong <- AssetQuantitiesLong
        LDV         asset_quantities_long_id    // Stack [AN, MAN, AQL]
        LDV         supply_long_id              // Stack [AN, MAN, AQL, SL = SupplyLong]
        JUPD        1   2   3                   // Stack [AN, MAN, AQL, SL_updated]
        STR         _SupplyLong                 // Stack [AN, MAN, AQS]
        POPN        1                           // Stack [AN, MAN]
        
        // Update Delta
        //
        // (Delta Long - Delta Short) = (Supply Long + Demand Short) - (Supply Short + Demand Long)
        //
        
        // Supply Long + Demand Short
        LDR         _SupplyLong
        LDV         demand_short_id
        ADD         1                           // Stack [AssetNames, MarketAssetNames, SupplyLong, DeltaLong]
        SWAP        1
        POPN        1                           // Stack [AssetNames, MarketAssetNames, DeltaLong]

        // Supply Short + Demand Long
        LDR         _SupplyShort
        LDV         demand_long_id
        ADD         1                           // Stack [AssetNames, MarketAssetNames, DeltaLong, SupplyShort, DeltaShort]
        SWAP        1
        POPN        1                           // Stack [AssetNames, MarketAssetNames, DeltaLong, DeltaShort]

        // Delta Long - Delta Short
        LDD         0                           // Stack [AssetNames, MarketAssetNames, DeltaLong, DeltaShort, DeltaShort]
        SSB         2                           // Stack [AssetNames, MarketAssetNames, DeltaLong, DeltaShort, RS = (DeltaShort s- DeltaLong)]
        STR         _DeltaShort                 // Stack [AssetNames, MarketAssetNames, DeltaLong, DeltaShort]
        SWAP        1                           // Stack [AssetNames, MarketAssetNames, DeltaShort, DeltaLong]
        SSB         1                           // Stack [AssetNames, MarketAssetNames, DeltaShort, RL = (DeltaLong s- DeltaShort)]
        STR         _DeltaLong                  // Stack [AssetNames, MarketAssetNames, DeltaShort]
        POPN        3                           // Stack []
        
        
        // =============================
        // * * * COMMIT NEW VALUES * * *
        // =============================

        // Store Demand
        LDM         _SupplyLong
        LDM         _SupplyShort
        STV         supply_short_id
        STV         supply_long_id

        // Store Delta
        LDM         _DeltaLong
        LDM         _DeltaShort
        STV         delta_short_id
        STV         delta_long_id

    }
}