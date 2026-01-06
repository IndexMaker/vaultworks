use abacus_macros::abacus;

/// Execute Transfer
/// 
pub fn execute_transfer(
    sender_bid_id: u128,
    sender_ask_id: u128,
    receiver_bid_id: u128,
    amount: u128,
) -> Vec<u8> {
    abacus! {
        // ====================================
        // * * * (TRY) COMPUTE NEW VALUES * * *
        // ====================================

        LDV     sender_bid_id   // [S_bid]
        UNPK                    // [C_rem, C_spent, ITP_mint]
        LDV     sender_ask_id   // [C_rem, C_spent, ITP_mint, S_ask]
        UNPK                    // [C_rem, C_spent, ITP_mint, ITP_rem, ITP_burn, C_wd]
        
        // Compute: ITP_mint_new = ITP_mint - ITP_amount
        //
        IMMS    amount          // [C_rem, C_spent, ITP_mint, ITP_rem, ITP_burn, C_wd, ITP_amount]
        SWAP    6               // [ITP_amount, C_spent, ITP_mint, ITP_rem, ITP_burn, C_wd, C_rem]
        STR     _SenderRemain   // [ITP_amount, C_spent, ITP_mint, ITP_rem, ITP_burn, C_wd]

        LDD     3               // [ITP_amount, C_spent, ITP_mint, ITP_rem, ITP_burn, C_wd, ITP_mint]
        SSB     6               // [ITP_amount, C_spent, ITP_mint, ITP_rem, ITP_burn, C_wd, ITP_mint_new = (ITP_mint - ITP_amount)]
        LDD     1               // [ITP_amount, C_spent, ITP_mint, ITP_rem, ITP_burn, C_wd, ITP_mint_new, ITP_mint_new]
        STR     _SenderMinted   // [ITP_amount, C_spent, ITP_mint, ITP_rem, ITP_burn, C_wd, ITP_mint_new]

        // Test: ITP_mint_new >= ITP_rem - ITP_burn
        //
        SUB     3               // [ITP_amount, C_spent, ITP_mint, ITP_rem, ITP_burn, C_wd, (ITP_mint_new - ITP_rem)]
        SUB     2               // [ITP_amount, C_spent, ITP_mint, ITP_rem, ITP_burn, C_wd, (ITP_mint_new - ITP_rem - ITP_burn)]
        POPN    4               // [ITP_amount, C_spent, ITP_mint]

        // Compute: k = ITP_amount / ITP_mint
        //          C_spent_amount = k * C_spent
        //          C_spent_new = C_spent - C_spent_amount
        //
        LDD     2               // [ITP_amount, C_spent, ITP_mint, ITP_amount]
        DIV     1               // [ITP_amount, C_spent, ITP_mint, k = (ITP_amount / ITP_mint)]
        MUL     2               // [ITP_amount, C_spent, ITP_mint, C_amount = (k * C_spent)]
        SWAP    2               // [ITP_amount, C_amount, ITP_mint, C_spent]
        SUB     2               // [ITP_amount, C_amount, ITP_mint, C_spent_new = (C_spent - C_amount)]
        STR     _SenderSpent    // [ITP_amount, C_amount, ITP_mint]

        LDV     receiver_bid_id // [ITP_amount, C_amount, ITP_mint, R_bid]
        UNPK                    // [ITP_amount, C_amount, ITP_mint, rC_rem, rC_spent, rITP_minted]
        ADD     5               // [ITP_amount, C_amount, ITP_mint, rC_rem, rC_spent, rITP_minted_new = (rITP_minted + ITP_amount)]
        STR     _ReceiverMinted // [ITP_amount, C_amount, ITP_mint, rC_rem, rC_spent]

        ADD     3               // [ITP_amount, C_amount, ITP_mint, rC_rem, rC_spent_new = (rC_spent + C_amount)]
        STR     _ReceiverSpent  // [ITP_amount, C_amount, ITP_mint, rC_rem]
        STR     _ReceiverRemain // [ITP_amount, C_amount, ITP_mint]
        POPN    3               // []
 
        // =============================
        // * * * COMMIT NEW VALUES * * *
        // =============================

        LDM     _SenderRemain
        LDM     _SenderSpent
        LDM     _SenderMinted
        PKV     3               // [(sC_remain, sC_spent, sITP_minted)]
        STV     sender_bid_id   // []

        LDM     _ReceiverRemain
        LDM     _ReceiverSpent
        LDM     _ReceiverMinted
        PKV     3               // [(rC_remain, rC_spent, rITP_minted)]
        STV     receiver_bid_id // []

    }
}
