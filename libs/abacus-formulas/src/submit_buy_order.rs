use abacus_macros::abacus;

/// Submit Buy Index Order
///
pub fn submit_buy_order(
    order_id: u128,
    vendor_order_id: u128,
    total_order_id: u128,
    collateral_added: u128,
    collateral_removed: u128,
) -> Vec<u8> {
    abacus! {
        // Load Index Order
        LDV         order_id                    // Stack: [Order = (Collateral, Spent, Minted)]
        LDV         vendor_order_id             // Stack: [Order, Vendor]
        LDV         total_order_id              // Stack: [Order, Vendor, Total]
        T           3                           // Stack: [Collateral, Spent, Minted]
        UNPK                                    // Stack: [Collateral, Spent, Minted_order, Minted_vendor, Minted_total]
        STR         _MintedTotal                // Stack: [Collateral, Spent, Minted_order, Minted_vendor]
        STR         _MintedVendor               // Stack: [Collateral, Spent, Minted_order]
        STR         _Minted                     // Stack: [Collateral, Spent]
        UNPK                                    // Stack: [Collateral, Spent_order, Spent_vendor, Spent_total]
        STR         _SpentTotal                 // Stack: [Collateral, Spent_order, Spent_vendor]
        STR         _SpentVendor                // Stack: [Collateral, Spent_order]
        STR         _Spent                      // Stack: [Collateral]

        // Compute Collateral += (Collateral Added - Collateral Removed)
        IMMS        collateral_added            // Stack: [Collateral, C.Added]
        SWAP        1                           // Stack: [C.Added, Collateral]
        ADD         1                           // Stack: [Collateral_old, Collateral_new = (Collateral_old + C.Added)]
        IMMS        collateral_removed          // Stack: [Collateral_old, Collateral_new, C.Removed]
        SWAP        1                           // Stack: [Collateral_old, C.Removed, Collateral_new]
        SUB         1                           // Stack: [Collateral_old, C.Removed, (Collateral_new - C.Removed)]
        UNPK                                    // Stack: [Collateral_old, C.Removed, C_order, C_vendor, C_total]
        STR         _CollateralTotal            // Stack: [Collateral_old, C.Removed, C_order, C_vendor]
        STR         _CollateralVendor           // Stack: [Collateral_old, C.Removed, C_order]
        STR         _Collateral                 // Stack: [Collateral_old, C.Removed]
        POPN        2                           // Stack: []

        LDM         _Collateral
        LDM         _CollateralVendor
        LDM         _CollateralTotal
        PKV         3

        LDM         _Spent
        LDM         _SpentVendor
        LDM         _SpentTotal
        PKV         3

        LDM         _Minted
        LDM         _MintedVendor
        LDM         _MintedTotal
        PKV         3

        // Store Updated Order
        T           3                               // Stack: [Order, Vendor, Total]
        STV         total_order_id                  // Stack: [Order, Vendor]
        STV         vendor_order_id                 // Stack: [Order]
        STV         order_id                        // Stack: []

    }
}
