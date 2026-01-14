use abacus_macros::abacus;

/// Update Margin
/// 
pub fn update_margin(
    asset_names_id: u128,
    asset_margin_id: u128,
    market_asset_names_id: u128,
    margin_id: u128,
) -> Result<Vec<u8>, Vec<u8>> {
    abacus! {
        // ====================================
        // * * * (TRY) COMPUTE NEW VALUES * * *
        // ====================================

        // Update Margin
        LDL         asset_names_id              // Stack [AN = AssetNames]
        LDL         market_asset_names_id       // Stack [AN = AssetNames, MAN = MarketAssetNames]
        LDV         asset_margin_id             // Stack [AN, MAN, AM]
        LDV         margin_id                   // Stack [AN, MAN, AM, M = Margin]
        JUPD        1   2   3                   // Stack [AN, MAN, AM, M_updated]
        STR         _Margin                     // Stack [AN, MAN, AM]
        POPN        1                           // Stack [AN, MAN]
        
        // =============================
        // * * * COMMIT NEW VALUES * * *
        // =============================

        // Store Margin
        LDM         _Margin
        STV         margin_id

    }
}