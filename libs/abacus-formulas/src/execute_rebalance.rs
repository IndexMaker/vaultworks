use abacus_macros::abacus;

/// Execute rebalance order
///
pub fn execute_rebalance(
    capacity_factor: u128,
    executed_assets_long_id: u128,
    executed_assets_short_id: u128,
    rebalance_asset_names_id: u128,
    rebalance_weights_long_id: u128,
    rebalance_weights_short_id: u128,
    market_asset_names_id: u128,
    supply_long_id: u128,
    supply_short_id: u128,
    demand_long_id: u128,
    demand_short_id: u128,
    delta_long_id: u128,
    delta_short_id: u128,
    margin_id: u128,
    asset_liquidity_id: u128,
) -> Result<Vec<u8>, Vec<u8>> {
    abacus! {
        // ====================================
        // * * * (TRY) COMPUTE NEW VALUES * * *
        // ====================================
        //
        //  1. Cap Rebalance w/ CapacityLimit
        //  2. Scale CapacityLimit w/ CapacityFactor
        //  3. Add Capped Rebalance to Delta
        //  4. Return Executed Asset Quantities
        //

        // Rebalance Long:  CapacityLimit = (DeltaLong + MIN(Margin - DeltaShort, AssetLiquidity))
        // Rebalance Short: CapacityLimit = (DeltaShort + MIN(Margin - DeltaLong, AssetLiquidity))
        LDL         rebalance_asset_names_id    // Stack: [AN = RebalanceAssetNames]
        LDL         market_asset_names_id       // Stack: [AN, MAN = MarketAssetNames]
        LDV         margin_id                   // Stack: [AN, MAN, M = Margin]
        JFLT        1   2                       // Stack: [AN, MAN, fM]
        LDV         asset_liquidity_id          // Stack: [AN, MAN, fM, AL = AssetLiquidity]
        JFLT        2   3                       // Stack: [AN, MAN, fM, fAL]
        LDV         delta_short_id              // Stack: [AN, MAN, fM, fAL, DS = DeltaShort]
        JFLT        3   4                       // Stack: [AN, MAN, fM, fDS]
        LDV         delta_long_id               // Stack: [AN, MAN, fM, fAL, fDS, DL = DeltaLong]
        JFLT        4   5                       // Stack: [AN, MAN, fM, fAL, fDS, fDL]
        LDD         3                           // Stack: [AN, MAN, fM, fAL, fDS, fDL, fM]
        SSB         1                           // Stack: [AN, MAN, fM, fAL, fDS, fDL, (fM - fDL)]
        SWAP        4                           // Stack: [AN, MAN, (fM - fDL), fAL, fDS, fDL, fM]
        SSB         2                           // Stack: [AN, MAN, (fM - fDL), fAL, fDS, fDL, (fM - fDS)]
        MIN         3                           // Stack: [AN, MAN, (fM - fDL), fAL, fDS, fDL, MIN(fM - fDS, fAL)]
        ADD         1                           // Stack: [AN, MAN, (fM - fDL), fAL, fDS, fDL, fDL + MIN(fM - fDS, fAL)]
        STR         _CapacityLimitLong          // Stack: [AN, MAN, (fM - fDL), fAL, fDS, fDL]
        SWAP        2                           // Stack: [AN, MAN, (fM - fDL), fDL, fDS, fAL]
        MIN         3                           // Stack: [AN, MAN, (fM - fDL), fDL, fDS, MIN(fM - fDL, fAL)]
        ADD         1                           // Stack: [AN, MAN, (fM - fDL), fDL, fDS, fDS + MIN(fM - fDL, fAL)]
        STR         _CapacityLimitShort         // Stack: [AN, MAN, (fM - fDL), fDL, fDS]
        POPN        3                           // Stack: [AN, MAN]

        IMMS        capacity_factor             // Stack: [AN, MAN, F]
        LDV         rebalance_weights_long_id   // Stack: [AN, MAN, F, RL]
        LDM         _CapacityLimitLong          // Stack: [AN, MAN, F, RL, CLL]
        MUL         2                           // Stack: [AN, MAN, F, RL, F * CLL]
        MIN         1                           // Stack: [AN, MAN, F, RL, DRL = MIN(RL, F * CLL)]
        SWAP        1                           // Stack: [AN, MAN, F, DRL, RL]
        SUB         1                           // Stack: [AN, MAN, F, DRL, RL_new = RL - DRL]
        STR         _RebalanceWeightsLong       // Stack: [AN, MAN, F, DRL]
        STR         _RebalanceDeltaLong         // Stack: [AN, MAN, F]

        LDV         rebalance_weights_short_id  // Stack: [AN, MAN, F, RS]
        LDM         _CapacityLimitShort         // Stack: [AN, MAN, F, RS, CLS]
        MUL         2                           // Stack: [AN, MAN, F, RS, F * CLS]
        MIN         1                           // Stack: [AN, MAN, F, RS, DRS = MIN(RS, F * CLS)]
        SWAP        1                           // Stack: [AN, MAN, F, DRS, RS]
        SUB         1                           // Stack: [AN, MAN, F, DRS, RS_new = RS - DRS]
        STR         _RebalanceWeightsShort      // Stack: [AN, MAN, F, DRS]
        STR         _RebalanceDeltaShort        // Stack: [AN, MAN, F]
        POPN        1                           // Stack: [AN, MAN]

        LDV         demand_long_id              // Stack: [AN, MAN, DL]
        JFLT        1   2                       // Stack: [AN, MAN, fDL]
        LDV         demand_short_id             // Stack: [AN, MAN, fDL, DS]
        JFLT        2   3                       // Stack: [AN, MAN, fDL, fDS]
        LDM         _RebalanceDeltaLong         // Stack: [AN, MAN, fDL, fDS, RDL]
        LDM         _RebalanceDeltaShort        // Stack: [AN, MAN, fDL, fDS, RDL, RDS]
        ADD         2                           // Stack: [AN, MAN, fDL, fDS, RDL, (RDS + fDS)]
        SWAP        1                           // Stack: [AN, MAN, fDL, fDS, (RDS + fDS), RDL]
        ADD         3                           // Stack: [AN, MAN, fDL, fDS, RS = (RDS + fDS), RL = (RDL + fDL)]
        LDD         1                           // Stack: [AN, MAN, fDL, fDS, RS, RL, RL]
        SSB         2                           // Stack: [AN, MAN, fDL, fDS, RS, RL, DL_new = (RL s-  RS)]
        SWAP        2                           // Stack: [AN, MAN, fDL, fDS, DL_new, RL, RS]
        SSB         1                           // Stack: [AN, MAN, fDL, fDS, DL_new, RL, DS_new = (RS s- RL)]
        STR         _DemandShort                // Stack: [AN, MAN, fDL, fDS, DL_new, RL]
        SWAP        1                           // Stack: [AN, MAN, fDL, fDS, RL, DL_new]
        STR         _DeltaLong                  // Stack: [AN, MAN, fDL, fDS, RL]
        POPN        3                           // Stack: [AN, MAN]

        // Update Delta
        //
        // (Delta Long - Delta Short) = (Supply Long + Demand Short) - (Supply Short + Demand Long)
        //

        // Delta Long = Supply Long + Demand Short
        LDV         supply_long_id              // Stack [AN, MAN, SL]
        JFLT        1   2                       // Stack [AN, MAN, fSL]
        LDR         _DemandShort                // Stack [AN, MAN, fSL, DS]
        ADD         1                           // Stack [AssetNames, MarketAssetNames, SupplyLong, DeltaLong]
        SWAP        1
        POPN        1                           // Stack [AssetNames, MarketAssetNames, DeltaLong]

        // Delta Short = Supply Short + Demand Long
        LDV         supply_short_id             // Stack [AN, MAN, SS]
        JFLT        1   2                       // Stack [AN, MAN, fSS]
        LDR         _DemandLong                 // Stack [AN, MAN, fSD, DL]
        ADD         1                           // Stack [AssetNames, MarketAssetNames, DeltaLong, SupplyShort, DeltaShort]
        SWAP        1
        POPN        1                           // Stack [AssetNames, MarketAssetNames, DeltaLong, DeltaShort]

        // Normalize Delta(Long|Short) = Delta Long - Delta Short
        LDD         0                           // Stack [AssetNames, MarketAssetNames, DeltaLong, DeltaShort, DeltaShort]
        SSB         2                           // Stack [AssetNames, MarketAssetNames, DeltaLong, DeltaShort, RS = (DeltaShort s- DeltaLong)]
        STR         _DeltaShort                 // Stack [AssetNames, MarketAssetNames, DeltaLong, DeltaShort]
        SWAP        1                           // Stack [AssetNames, MarketAssetNames, DeltaShort, DeltaLong]
        SSB         1                           // Stack [AssetNames, MarketAssetNames, DeltaShort, RL = (DeltaLong s- DeltaShort)]
        STR         _DeltaLong                  // Stack [AssetNames, MarketAssetNames, DeltaShort]
        POPN        1                           // Stack [AN, MAN]


        // =============================
        // * * * COMMIT NEW VALUES * * *
        // =============================

        // Expand & Store Demand
        LDM         _DemandLong             // Stack [AN, MAN, fDL]
        ZEROS       1                       // Stack [AN, MAN, fDL, Z]
        JUPD        1   2   3               // Stack [AN, MAN, fDL, DL]
        STV         demand_long_id

        LDM         _DemandShort            // Stack [AN, MAN, fDS]
        ZEROS       1                       // Stack [AN, MAN, fDS, Z]
        JUPD        1   2   3               // Stack [AN, MAN, fDS, DS]
        STV         demand_short_id

        // Expand & Store Delta
        LDM         _DeltaLong
        ZEROS       1
        JUPD        1   2   3
        STV         delta_short_id
        
        LDM         _DeltaShort
        ZEROS       1
        JUPD        1   2   3
        STV         delta_long_id

        // Storte executed assets quantities (Buy|Sell)
        LDM         _RebalanceDeltaLong
        LDM         _RebalanceDeltaShort
        STV         executed_assets_short_id
        STV         executed_assets_long_id

    }
}
