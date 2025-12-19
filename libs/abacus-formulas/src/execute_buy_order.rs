use abacus_macros::abacus;

/// Execute Buy Index Order
/// 
pub fn execute_buy_order(
    order_id: u128,
    collateral_added: u128,
    collateral_removed: u128,
    max_order_size: u128,
    executed_index_quantities_id: u128,
    executed_asset_quantities_id: u128,
    asset_names_id: u128,
    asset_weights_id: u128,
    index_quote_id: u128,
    market_asset_names_id: u128,
    supply_long_id: u128,
    supply_short_id: u128,
    demand_long_id: u128,
    demand_short_id: u128,
    delta_long_id: u128,
    delta_short_id: u128,
    margin_id: u128,
    asset_contribution_fractions_id: u128,
    solve_quadratic_id: u128,
) -> Vec<u8> {
    abacus! {
        // ====================================
        // * * * (TRY) COMPUTE NEW VALUES * * *
        // ====================================

        // Load Weights
        LDV         asset_weights_id            // Stack: [AssetWeights]
        STR         _Weights                    // Stack: []

        // Load Index Order
        LDV         order_id                    // Stack: [Order = (Collateral, Spent, Minted)] 
        UNPK                                    // Stack: [Collateral, Spent, Minted]
        STR         _Minted                     // Stack: [Collateral, Spent]
        STR         _Spent                      // Stack: [Collateral]

        // Compute Collateral += (Collateral Added - Collateral Removed)
        IMMS        collateral_added            // Stack: [Collateral, C.Added]
        ADD         1                           // Stack: [Collateral_old, Collateral_new = (Collateral_old + C.Added)]
        IMMS        collateral_removed          // Stack: [Collateral_old, Collateral_new, C.Removed]
        SWAP        1                           // Stack: [Collateral_old, C.Removed, Collateral_new]
        SUB         1                           // Stack: [Collateral_old, C.Removed, (Collateral_new - C.Removed)]
        STR         _Collateral                 // Stack: [Collateral_old, C.Removed]
        POPN        2                           // Stack: []

        // Store updated Order = (Collateral, Spent, Minted)
        //
        // Note that if we fail Margin Test we still want to keep user's order updated.
        //
        LDR         _Collateral
        LDR         _Spent
        LDR         _Minted
        PKV         3
        STV         order_id

        // Compute Index Quantity
        LDV         index_quote_id              // Stack: [Quote = (Capacity, Price, Slope)]
        UNPK                                    // Stack: [Capacity, Price, Slope]
        SWAP        2                           // Stack: [Slope, Price, Capacity]
        STR         _Capacity                   // Stack: [Slope, Price]
        STR         _Price                      // Stack: [Slope]
        STR         _Slope                      // Stack: []
        
        // Solve Quadratic: S * Q^2 + P * Q - C = 0
        IMMS        max_order_size              // Stack: [MaxOrderSize]
        LDR         _Slope                      // Stack: [MaxOrderSize, Slope]
        LDR         _Price                      // Stack: [MaxOrderSize, Slope, Price]
        LDR         _Collateral                 // Stack: [MaxOrderSize, Slope, Price, Collateral]
        MIN         3                           // Stack: [MaxOrderSize, Slope, Price, CappedCollateral]
        B           solve_quadratic_id  3  1  4 // Stack: [MaxOrderSize, IndexQuantity]
        STR         _IndexQuantity              // Stack: [MaxOrderSize]
        POPN        1                           // Stack: []

        // Compute CapacityLimit = MIN( (DeltaLong + MIN(Margin - DeltaShort, Capacity * AssetWeights)) / AssetWeights)
        LDL         asset_names_id              // Stack: [AssetNames]
        LDL         market_asset_names_id       // Stack: [AN = AssetNames, MAN = MarketAssetNames]
        LDV         asset_contribution_fractions_id // Stack: [AN, MAN, ACF = AssetContributionFractions]
        LDV         margin_id                   // Stack: [AN, MAN, ACF, M = Margin]
        LDV         delta_long_id               // Stack: [AN, MAN, ACF, M, DL = DeltaLong]
        JFLT        3   4                       // Stack: [AN, MAN, ACF, M, fDL]
        LDV         delta_short_id              // Stack: [AN, MAN, ACF, M, fDL, DS = DeltaShort]
        SWAP        2                           // Stack: [AN, MAN, ACF, DS, fDL, M]
        SSB         2                           // Stack: [AN, MAN, ACF, DS, fDL, M_DS = M s- DS]
        JFLT        4   5                       // Stack: [AN, MAN, ACF, DS, fDL, fM_DS]
        LDR         _Weights                    // Stack: [AN, MAN, ACF, DS, fDL, fM_DS, W = AssetWeights]
        LDM         _Capacity                   // Stack: [AN, MAN, ACF, DS, fDL, fM_DS, W, Cap]
        LDD         1                           // Stack: [AN, MAN, ACF, DS, fDL, fM_DS, W, Cap, W]
        MUL         1                           // Stack: [AN, MAN, ACF, DS, fDL, fM_DS, W, Cap, Cap_W = Cap * W]
        MIN         3                           // Stack: [AN, MAN, ACF, DS, fDL, fM_DS, W, Cap, MA = MIN(fM_DS, Cap_W)]
        ADD         4                           // Stack: [AN, MAN, ACF, DS, fDL, fM_DS, W, Cap, L = MA + fDL]
        DIV         2                           // Stack: [AN, MAN, ACF, DS, fDL, fM_DS, W, Cap, CL_vec = L / W]
        MUL         6                           // Stack: [AN, MAN, ACF, DS, fDL, fM_DS, W, Cap, CL_vec_acf = CL_vec * ACF]
        VMIN                                    // Stack: [AN, MAN, ACF, DS, fDL, fM_DS, W, Cap, CL = VMIN(CL_vec_acf)]
        SWAP        6                           // Stack: [AN, MAN, CL, DS, fDL, fM_DS, W, Cap, ACF]
        POPN        6                           // Stack: [AN, MAN, CL]
        SWAP        2                           // Stack: [CL, MAN, AN]
        STR         _AssetNames                 // Stack: [CL, MAN]
        STR         _MarketAssetNames           // Stack: [CapacityLimit = CL]
        
        // Cap Index Quantity with Capacity
        LDR         _IndexQuantity              // Stack: [CapacityLimit, IndexQuantity]
        MIN         1                           // Stack: [CapacityLimit, CIQ = MIN(Capacity, IndexQuantity)]
        STR         _CappedIndexQuantity        // Stack: [CapacityLimit ]
        POPN        1                           // Stack: []

        // Generate Individual Asset Orders (compute asset quantities)
        LDR         _CappedIndexQuantity        // Stack: [CIQ]
        LDM         _Weights                    // Stack: [CIQ, AssetWeights]
        MUL         1                           // Stack: [CIQ, AssetQuantities]
        
        STR         _AssetQuantities            // Stack: [CIQ]
        POPN        1                           // Stack: []

        // Match Market: Update Demand and Delta
        LDM         _AssetNames                 // Stack [AssetNames]
        LDM         _MarketAssetNames           // Stack [AssetNames, MarketAssetNames]
        
        // Compute Demand Short = MAX(Demand Short - Asset Quantities, 0)
        LDV         demand_short_id             // Stack [AssetNames, MarketAssetNames, DS_old]
        LDR         _AssetQuantities            // Stack [AssetNames, MarketAssetNames, DS_old, AQ]
        LDD         1                           // Stack [AssetNames, MarketAssetNames, DS_old, AQ, DS_old]
        JFLT        3   4                       // Stack [AssetNames, MarketAssetNames, DS_old, AQ, fDS_old]
        LDD         0                           // Stack [AssetNames, MarketAssetNames, DS_old, AQ, fDS_old, fDS_old]
        SSB         2                           // Stack [AssetNames, MarketAssetNames, DS_old, AQ, fDS_old, fDS_new = (fDS_old s- AQ)]
        SWAP        3                           // Stack [AssetNames, MarketAssetNames, fDS_new, AQ, fDS_old, DS_old]
        JUPD        3   4   5                   // Stack [AssetNames, MarketAssetNames, fDS_new, AQ, fDS_old, DS_new]
        SWAP        3                           // Stack [AssetNames, MarketAssetNames, DS_new, AQ, fDS_old, fDS_new]
        POPN        1                           // Stack [AssetNames, MarketAssetNames, DS_new, AQ, fDS_old]

        // Compute Demand Long += MAX(Asset Quantities - Demand Short, 0)
        SWAP        1                           // Stack [AssetNames, MarketAssetNames, DS_new, fDS_old, AQ]
        SSB         1                           // Stack [AssetNames, MarketAssetNames, DS_new, fDS_old, dAQ = (AQ s- fDS_old)]
        LDV         demand_long_id              // Stack [AssetNames, MarketAssetNames, DS_new, fDS_old, dAQ, DL_old]
        JADD        1   4   5                   // Stack [AssetNames, MarketAssetNames, DS_new, fDS_old, dAQ, DL_new = (DL_old j+ dAQ)]
        SWAP        2                           // Stack [AssetNames, MarketAssetNames, DS_new, DL_new, dAQ, fDS_old]
        POPN        2                           // Stack [AssetNames, MarketAssetNames, DS_new, DL_new]
        STR         _DemandLong                 // Stack [AssetNames, MarketAssetNames, DS_new]
        STR         _DemandShort                // Stack [AssetNames, MarketAssetNames]
        
        // Update Delta
        //
        // (Delta Long - Delta Short) = (Supply Long + Demand Short) - (Supply Short + Demand Long)
        //
        
        // Supply Long + Demand Short
        LDV         supply_long_id
        LDR         _DemandShort
        ADD         1                           // Stack [AssetNames, MarketAssetNames, SupplyLong, DeltaLong]
        SWAP        1
        POPN        1                           // Stack [AssetNames, MarketAssetNames, DeltaLong]

        // Supply Short + Demand Long
        LDV         supply_short_id
        LDR         _DemandLong
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
        LDM         _DemandLong
        LDM         _DemandShort
        STV         demand_short_id
        STV         demand_long_id

        // Store Delta
        LDM         _DeltaLong
        LDM         _DeltaShort
        STV         delta_short_id
        STV         delta_long_id

        // Compute Collateral Spent
        LDR         _CappedIndexQuantity            // Stack: [CIQ]
        LDM         _Slope                          // Stack: [CIQ, Slope]
        MUL         1                               // Stack: [CIQ, SQ = (S * Q)]
        LDM         _Price                          // Stack: [CIQ, SQ, Price]
        ADD         1                               // Stack: [CIQ, SQ, EP = (SQ + Price)]
        SWAP        1                               // Stack: [CIQ, EP, SQ]
        POPN        1                               // Stack: [CIQ, EP] 
        MUL         1                               // Stack: [CIQ, CS = (CIQ * EP)]
        
        // Compute Order Remaining Collateral 
        LDM         _Collateral                     // Stack: [CIQ, CS, C]
        SSB         1                               // Stack: [CIQ, CS, CR = (C - CS)]
        SWAP        1                               // Stack: [CIQ, CR, CS]

        // Compute Order Spent Collateral
        LDM         _Spent                          // Stack: [CIQ, CR, CS, CS_old]
        ADD         1                               // Stack: [CIQ, CR, CS, CS_new = (CS_old + CS)]
        SWAP        1                               // Stack: [CIQ, CR, CS_new, CS]
        POPN        1                               // Stack: [CIQ, CR, CS_new]
        SWAP        1                               // Stack: [CIQ, CS_new, CR]
        SWAP        2                               // Stack: [CR, CS_new, CIQ]

        // Store Updated Order 
        PKV         3                               // Stack: [(CR, CS_new, CIQ)]
        STV         order_id                        // Stack: []


        // Store Executed Index Quantity and Remaining Quantity
        LDM         _CappedIndexQuantity            // Stack: [CIQ]
        LDM         _IndexQuantity                  // Stack: [CIQ, IndexQuantity]
        SUB         1                               // Stack: [CIQ, RIQ = (IndexQuantity - CIQ)]
        PKV         2                               // Stack: [(CIQ, RIQ)]
        STV         executed_index_quantities_id    // Stack: []
        
        // Store Executed Asset Quantities
        LDM         _AssetQuantities
        STV         executed_asset_quantities_id
    }
}
