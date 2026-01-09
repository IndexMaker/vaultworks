use abacus_macros::abacus;

/// Submit Sell Index Order
///
pub fn submit_sell_order(
    order_id: u128,
    vendor_order_id: u128,
    total_order_id: u128,
    collateral_added: u128,
    collateral_removed: u128,
) -> Vec<u8> {
    abacus! {
        // Load Index Order
        LDV         order_id                    // Stack: [Order = (Collateral, Burned, Withdrawn)]
        LDV         vendor_order_id             // Stack: [Order, Vendor]
        LDV         total_order_id              // Stack: [Order, Vendor, Total]
        T           3                           // Stack: [Collateral, Burned, Withdrawn]
        UNPK                                    // Stack: [Collateral, Burned, Withdrawn_order, Withdrawn_vendor, Withdrawn_total]
        STR         _WithdrawnTotal             // Stack: [Collateral, Burned, Withdrawn_order, Withdrawn_vendor]
        STR         _WithdrawnVendor            // Stack: [Collateral, Burned, Withdrawn_order]
        STR         _Withdrawn                  // Stack: [Collateral, Burned]
        UNPK                                    // Stack: [Collateral, Burned_order, Burned_vendor, Burned_total]
        STR         _BurnedTotal                // Stack: [Collateral, Burned_order, Burned_vendor]
        STR         _BurnedVendor               // Stack: [Collateral, Burned_order]
        STR         _Burned                     // Stack: [Collateral]

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

        LDM         _Burned
        LDM         _BurnedVendor
        LDM         _BurnedTotal
        PKV         3

        LDM         _Withdrawn
        LDM         _WithdrawnVendor
        LDM         _WithdrawnTotal
        PKV         3

        // Store Updated Order
        T           3                               // Stack: [Order, Vendor, Total]
        STV         total_order_id                  // Stack: [Order, Vendor]
        STV         vendor_order_id                 // Stack: [Order]
        STV         order_id                        // Stack: []

    }
}
