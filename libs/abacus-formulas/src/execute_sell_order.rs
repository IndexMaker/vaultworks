use abacus_macros::abacus;

/// Execute Sell Index Order
/// 
pub fn execute_sell_order(
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
        // Load Weights
        LDV         asset_weights_id            // Stack: [AssetWeights]
        STR         _Weights                    // Stack: []

        // Load Index Order
        LDV         order_id                    // Stack: [Order = (Collateral, Burned, Withdrawn)] 
        UNPK                                    // Stack: [Collateral, Burned, Withdrawn]
        STR         _Withdrawn                  // Stack: [Collateral, Burned]
        STR         _Burned                     // Stack: [Collateral]

        // Compute Collateral += (Collateral Added - Collateral Removed)
        IMMS        collateral_added            // Stack: [Collateral, C.Added]
        ADD         1                           // Stack: [Collateral_old, Collateral_new = (Collateral_old + C.Added)]
        IMMS        collateral_removed          // Stack: [Collateral_old, Collateral_new, C.Removed]
        SWAP        1                           // Stack: [Collateral_old, C.Removed, Collateral_new]
        SUB         1                           // Stack: [Collateral_old, C.Removed, (Collateral_new - C.Removed)]
        STR         _Collateral                 // Stack: [Collateral_old, C.Removed]
        POPN        2                           // Stack: []

        // Store updated Order = (Collateral, Burned, Withdrawn)
        //
        // Note that if we fail Margin Test we still want to keep user's order updated.
        //
        LDR         _Collateral
        LDR         _Burned
        LDR         _Withdrawn
        PKV         3
        STV         order_id

        // Compute Index Quantity
        LDV         index_quote_id              // Stack: [Quote = (Capacity, Price, Slope)]
        UNPK                                    // Stack: [Capacity, Price, Slope]
        SWAP        2                           // Stack: [Slope, Price, Capacity]
        STR         _Capacity                   // Stack: [Slope, Price]
        STR         _Price                      // Stack: [Slope]
        STR         _Slope                      // Stack: []

        // Compute CapacityLimit = MIN( (DeltaShort + MIN(Margin - DeltaLong, Capacity * AssetWeights)) / AssetWeights)
        LDL         asset_names_id              // Stack: [AssetNames]
        LDL         market_asset_names_id       // Stack: [AN = AssetNames, MAN = MarketAssetNames]
        LDV         asset_contribution_fractions_id // Stack: [AN, MAN, ACF = AssetContributionFractions]
        LDV         margin_id                   // Stack: [AN, MAN, ACF, M = Margin]
        LDV         delta_short_id              // Stack: [AN, MAN, ACF, M, DS = DeltaShort]
        JFLT        3   4                       // Stack: [AN, MAN, ACF, M, fDS]
        LDV         delta_long_id               // Stack: [AN, MAN, ACF, M, fDS, DL = DeltaLong]
        SWAP        2                           // Stack: [AN, MAN, ACF, DL, fDS, M]
        SSB         2                           // Stack: [AN, MAN, ACF, DL, fDS, M_DL = M s- DL]
        JFLT        4   5                       // Stack: [AN, MAN, ACF, DL, fDS, fM_DL]
        LDR         _Weights                    // Stack: [AN, MAN, ACF, DL, fDS, fM_DL, W = AssetWeights]
        LDM         _Capacity                   // Stack: [AN, MAN, ACF, DL, fDS, fM_DL, W, Cap]
        LDD         1                           // Stack: [AN, MAN, ACF, DL, fDS, fM_DL, W, Cap, W]
        MUL         1                           // Stack: [AN, MAN, ACF, DL, fDS, fM_DL, W, Cap, Cap_W = Cap * W]
        MIN         3                           // Stack: [AN, MAN, ACF, DL, fDS, fM_DL, W, Cap, MA = MIN(fM_DL, Cap_W)]
        ADD         4                           // Stack: [AN, MAN, ACF, DL, fDS, fM_DL, W, Cap, L = MA + fDS]
        DIV         2                           // Stack: [AN, MAN, ACF, DL, fDS, fM_DL, W, Cap, CL_vec = L / W]
        MUL         6                           // Stack: [AN, MAN, ACF, DL, fDS, fM_DL, W, Cap, CL_vec_acf = CL_vec * ACF]
        VMIN                                    // Stack: [AN, MAN, ACF, DL, fDS, fM_DL, W, Cap, CL = VMIN(CL_vec_acf)]
        SWAP        6                           // Stack: [AN, MAN, CL, DL, fDS, fM_DL, W, Cap, ACF]
        POPN        6                           // Stack: [AN, MAN, CL]
        SWAP        2                           // Stack: [CL, MAN, AN]
        STR         _AssetNames                 // Stack: [CL, MAN]
        STR         _MarketAssetNames           // Stack: [CapacityLimit = CL]

        // Compute WithdrawAmount = C * (P + C * S)
        //
        // Note that Collateral (C) is capped by CapacityLimit (CL), and then
        // we compute WithdrawAmount, which is capped by MaxOrderSize (M)
        //
        LDR         _Collateral                 // Stack: [CL, Collateral]
        MIN         1                           // Stack: [CL, C = MIN(CL, Collateral)]
        IMMS        max_order_size              // Stack: [CL, C, M = MaxOrderSize]
        SWAP        1                           // Stack: [CL, M, C]
        LDR         _Slope                      // Stack: [CL, M, C, S = Slope]
        MUL         1                           // Stack: [CL, M, C, SC = S * C]
        LDR         _Price                      // Stack: [CL, M, C, SC, P]
        SSB         1                           // Stack: [CL, M, C, SC, P - SC]
        MUL         2                           // Stack: [CL, M, C, SC, W = C * (P + SC)]
        MIN         3                           // Stack: [CL, M, C, SC, WC = MIN(W, M)]
        STR         _WithdrawAmount             // Stack: [CL, M, C]
        POPN        3                           // Stack: []

        // Solve Quadratic: -S * Q^2 + P * Q - C = 0
        LDR         _Slope                      // Stack: [MaxOrderSize, Slope]
        LDR         _Price                      // Stack: [MaxOrderSize, Slope, Price]
        LDR         _WithdrawAmount             // Stack: [MaxOrderSize, Slope, Price, WithdrawAmount]
        B           solve_quadratic_id  3  1  4 // Stack: [MaxOrderSize, CappedIndexQuantity]
        STR         _CappedIndexQuantity        // Stack: [MaxOrderSize]
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
        
        // Compute Demand Long = MAX(Demand Long - Asset Quantities, 0)
        LDV         demand_long_id              // Stack [AssetNames, MarketAssetNames, DL_old]
        LDR         _AssetQuantities            // Stack [AssetNames, MarketAssetNames, DL_old, AQ]
        LDD         1                           // Stack [AssetNames, MarketAssetNames, DL_old, AQ, DL_old]
        JFLT        3   4                       // Stack [AssetNames, MarketAssetNames, DL_old, AQ, fDL_old]
        LDD         0                           // Stack [AssetNames, MarketAssetNames, DL_old, AQ, fDL_old, fDL_old]
        SSB         2                           // Stack [AssetNames, MarketAssetNames, DL_old, AQ, fDL_old, fDL_new = (fDL_old s- AQ)]
        SWAP        3                           // Stack [AssetNames, MarketAssetNames, fDL_new, AQ, fDL_old, DL_old]
        JUPD        3   4   5                   // Stack [AssetNames, MarketAssetNames, fDL_new, AQ, fDL_old, DL_new]
        SWAP        3                           // Stack [AssetNames, MarketAssetNames, DL_new, AQ, fDL_old, fDL_new]
        POPN        1                           // Stack [AssetNames, MarketAssetNames, DL_new, AQ, fDL_old]

        // Compute Demand Short += MAX(Asset Quantities - Demand Long, 0)
        SWAP        1                           // Stack [AssetNames, MarketAssetNames, DS_new, fDS_old, AQ]
        SSB         1                           // Stack [AssetNames, MarketAssetNames, DS_new, fDS_old, dAQ = (AQ s- fDS_old)]
        LDV         demand_short_id             // Stack [AssetNames, MarketAssetNames, DS_new, fDS_old, dAQ, DL_old]
        JADD        1   4   5                   // Stack [AssetNames, MarketAssetNames, DS_new, fDS_old, dAQ, DL_new = (DL_old j+ dAQ)]
        SWAP        2                           // Stack [AssetNames, MarketAssetNames, DS_new, DL_new, dAQ, fDS_old]
        POPN        2                           // Stack [AssetNames, MarketAssetNames, DS_new, DL_new]
        STR         _DemandShort                // Stack [AssetNames, MarketAssetNames, DS_new]
        STR         _DemandLong                 // Stack [AssetNames, MarketAssetNames]
        
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

        // Compute order vector (Collateral, Burned, Withdrawn)
        LDM         _WithdrawAmount             // Stack: [W = WithdrawAmount]
        LDR         _CappedIndexQuantity        // Stack: [W, CIQ = CappedIndexQuantity]
        LDR         _Collateral                 // Stack: [W, CIQ, C_old]
        SSB         1                           // Stack: [W, CIQ, C_new = C_old - CIQ]
        LDM         _Burned                     // Stack: [W, CIQ. C_new, B_old = Burned]
        ADD         2                           // Stack: [W, CIQ, C_new, B_new = B_old + CIQ]
        SWAP        2                           // Stack: [W, S_new, C_new, CIQ]
        POPN        1                           // Stack: [W, S_new, C_new]
        SWAP        2                           // Stack: [C_new, S_new, W]
        LDM         _Withdrawn                  // Stack: [C_new, S_new, W, W_old]
        ADD         1                           // Stack: [C_new, S_new, W_new, W]
        POPN        1                           // Stack: [C_new, S_new, W_new]
        PKV         3                           // Stack: [(C_new, S_new, W_new)]
        STV         order_id                    // Stack: []

        // Store Executed Index Quantity and Remaining Quantity
        LDM         _CappedIndexQuantity            // Stack: [CIQ]
        LDM         _Collateral                     // Stack: [CIQ, C_old]
        SUB         1                               // Stack: [CIQ, C_old - CIQ]
        PKV         2                               // Stack: [(CIQ, RIQ)]
        STV         executed_index_quantities_id    // Stack: []

        // Store Executed Asset Quantities
        LDM         _AssetQuantities
        STV         executed_asset_quantities_id
    }
}