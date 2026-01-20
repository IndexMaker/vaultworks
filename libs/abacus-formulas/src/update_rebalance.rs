use abacus_macros::abacus;

/// Compute and update rebalance vectors
/// 
pub fn update_rebalance(
    total_bid_id: u128,
    total_ask_id: u128,
    old_asset_names_id: u128,
    old_asset_weights_id: u128,
    new_asset_names_id: u128,
    new_asset_weights_id: u128,
    rebalance_asset_names_id: u128,
    rebalance_weights_long_id: u128,
    rebalance_weights_short_id: u128,
) -> Result<Vec<u8>, Vec<u8>> {
    abacus! {
        // Compute total supply
        LDV     total_bid_id                    // [T_bid]
        UNPK                                    // [C_bid, S_bid, M_bid]
        LDV     total_ask_id                    // [C_bid, S_bid, M_bid, T_ask]
        UNPK                                    // [C_bid, S_bid, M_bid, C_ask, S_ask, M_ask]
        SWAP    3                               // [C_bid, S_bid, M_ask, C_ask, S_ask, M_bid]
        SUB     1                               // [C_bid, S_bid, M_ask, C_ask, S_ask, M_bid - S_ask]
        SUB     2                               // [C_bid, S_bid, M_ask, C_ask, S_ask, T_supply]
        STR     _TotalSupply                    // [C_bid, S_bid, M_ask, C_ask, S_ask]
        POPN    5                               // []

        // Compute common Asset Names
        LDL     old_asset_names_id              // [AN_old]
        LDL     new_asset_names_id              // [AN_old, AN_new]
        LDV     rebalance_asset_names_id        // [AN_old, AN_new, AN_re]
        LDD     0                               // [AN_old, AN_new, AN_re, AN_re]
        LUNION  2                               // [AN_old, AN_new, AN_re, AN_uni_1]
        LUNION  3                               // [AN_old, AN_new, AN_re, AN_uni]
        ZEROS   0                               // [AN_old, AN_new, AN_re, AN_uni, Z_uni]

        // Expand Weights vectors to common Asset Names
        LDV     old_asset_weights_id            // [AN_old, AN_new, AN_re, AN_uni, Z_uni, W_old]
        LDD     1                               // [AN_old, AN_new, AN_re, AN_uni, Z_uni, W_old, Z_uni]
        JUPD    1   3   6                       // [AN_old, AN_new, AN_re, AN_uni, Z_uni, W_old, W_old_uni]
        LDV     new_asset_weights_id            // [AN_old, AN_new, AN_re, AN_uni, Z_uni, W_old, W_old_uni, W_new]
        SWAP    3                               // [AN_old, AN_new, AN_re, AN_uni, W_new, W_old, W_old_uni, Z_uni]
        JUPD    3   4   6                       // [AN_old, AN_new, AN_re, AN_uni, W_new, W_old, W_old_uni, W_new_uni]
        
        // Compute: Weights Delta = New Weights - Old Weights
        LDD     1                               // [AN_old, AN_new, AN_re, AN_uni, W_new, W_old, W_old_uni, W_new_uni, W_old_uni]
        SSB     1                               // [AN_old, AN_new, AN_re, AN_uni, W_new, W_old, W_old_uni, W_new_uni, dW_short = (W_old_uni s- W_new_uni)]
        STR     _DeltaWeightsShort              // [AN_old, AN_new, AN_re, AN_uni, W_new, W_old, W_old_uni, W_new_uni]
        SSB     1                               // [AN_old, AN_new, AN_re, AN_uni, W_new, W_old, W_old_uni, dW_long = (W_new_uni s- W_old_uni)]
        STR     _DeltaWeightsLong               // [AN_old, AN_new, AN_re, AN_uni, W_new, W_old, W_old_uni]
        POPN    3                               // [AN_old, AN_new, AN_re, AN_uni]

        // Expand Rebalance Long vector to common Asset Names
        LDV     rebalance_weights_long_id       // [AN_old, AN_new, AN_re, AN_uni, W_re_long]
        ZEROS   1                               // [AN_old, AN_new, AN_re, AN_uni, W_re_long, Z_re_uni]
        JUPD    1   2   3                       // [AN_old, AN_new, AN_re, AN_uni, W_re_long, W_re_uni]
        STR     _RebalanceWeightsLong           // [AN_old, AN_new, AN_re, AN_uni, W_re_long]
        POPN    1                               // [AN_old, AN_new, AN_re, AN_uni]

        // Expand Rebalance Short vector to common Asset Names
        LDV     rebalance_weights_short_id      // [AN_old, AN_new, AN_re, AN_uni, W_re_short]
        ZEROS   1                               // [AN_old, AN_new, AN_re, AN_uni, W_re_short, Z_re_uni]
        JUPD    1   2   3                       // [AN_old, AN_new, AN_re, AN_uni, W_re_short, W_re_uni]
        STR     _RebalanceWeightsShort          // [AN_old, AN_new, AN_re, AN_uni, W_re_short]
        POPN    1                               // [AN_old, AN_new, AN_re, AN_uni]
        STR     _RebalanceAssetNames            // [AN_old, AN_new, AN_re]
        POPN    3                               // []

        // Compute: New Rebalance Weights = Rebalance Weights + (Total Supply * Weights Delta)
        LDR     _TotalSupply                    // [T_supply]
        LDR     _RebalanceWeightsLong           // [T_supply, W_re_long]
        LDR     _DeltaWeightsLong               // [T_supply, W_re_long, dW_long]
        MUL     2                               // [T_supply, W_re_long, (dW_long * T_supply)]
        ADD     1                               // [T_supply, W_re_long, W_re_long_new = (dW_long * T_supply + W_re_long)]
        LDR     _RebalanceWeightsShort          // [T_supply, W_re_long, W_re_long_new, W_re_short]
        LDR     _DeltaWeightsShort              // [T_supply, W_re_long, W_re_long_new, W_re_short, dW_short]
        MUL     4                               // [T_supply, W_re_long, W_re_long_new, W_re_short, (dW_short * T_supply)]
        ADD     1                               // [T_supply, W_re_long, W_re_long_new, W_re_short, W_re_short_new = (dW_short * T_supply + W_re_short)]

        // Compute: Rebalance Weights Long - Rebalance Weights Short
        SWAP    1                               // [T_supply, W_re_long, W_re_long_new, W_re_short_new, W_re_short]
        POPN    1                               // [T_supply, W_re_long, W_re_long_new, W_re_short_new]
        LDD     1                               // [T_supply, W_re_long, W_re_long_new, W_re_short_new, W_re_long_new]
        SSB     1                               // [T_supply, W_re_long, W_re_long_new, W_re_short_new, R_long = W_re_long_new s- W_re_short_new]
        SWAP    1                               // [T_supply, W_re_long, W_re_long_new, R_long, W_re_short_new]
        SSB     2                               // [T_supply, W_re_long, W_re_long_new, R_long, R_short = W_re_short_new s- W_re_long_new]

        // Store new rebalance weights
        STV     rebalance_weights_short_id
        STV     rebalance_weights_long_id
    }
}