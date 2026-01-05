use alloc::vec::Vec;

use common::{amount::Amount, vector::Vector};

pub const ORDER_REMAIN_OFFSET: usize = 0;
pub const ORDER_SPENT_OFFSET: usize = 1;
pub const ORDER_REALIZED_OFFSET: usize = 2;

pub const QUOTE_CAPACITY_OFFSET: usize = 0;
pub const QUOTE_PRICE_OFFSET: usize = 1;
pub const QUOTE_SLOPE_OFFSET: usize = 2;

pub const REPORT_EXECUTED: usize = 0;
pub const REPORT_REMAINING: usize = 1;

pub struct Order;

impl Order {
    pub fn tell_total_from_vec(
        sender_bid_bytes: Vec<u8>,
        sender_ask_bytes: Vec<u8>,
    ) -> Result<Amount, Vec<u8>> {
        let sender_bid = Vector::from_vec(sender_bid_bytes);
        let sender_ask = Vector::from_vec(sender_ask_bytes);
        Self::tell_total(sender_bid, sender_ask)
    }

    /// Total amount of ITP in existence
    /// 
    /// This includes all ITP that was minted and not burned.
    /// 
    /// This is total amount, which includes active balance that can be used
    /// plus any balance locked for redeem, but not redeemed yet.
    /// 
    pub fn tell_total(sender_bid: Vector, sender_ask: Vector) -> Result<Amount, Vec<u8>> {
        let sender_itp_minted = sender_bid.data[ORDER_REALIZED_OFFSET];
        let sender_itp_burned = sender_ask.data[ORDER_SPENT_OFFSET];

        let sender_balance = sender_itp_minted
            .checked_sub(sender_itp_burned)
            .ok_or_else(|| b"MathUnderflow (minted < (redeem + burned)")?;

        Ok(sender_balance)
    }

    pub fn tell_available_from_vec(
        sender_bid_bytes: Vec<u8>,
        sender_ask_bytes: Vec<u8>,
    ) -> Result<Amount, Vec<u8>> {
        let sender_bid = Vector::from_vec(sender_bid_bytes);
        let sender_ask = Vector::from_vec(sender_ask_bytes);
        Self::tell_available(sender_bid, sender_ask)
    }

    /// Available amount of ITP
    /// 
    /// This includes all ITP that was minted and neither burned nor pending redeem.
    /// 
    /// This is an active balance of an account, that can be used.
    /// 
    pub fn tell_available(sender_bid: Vector, sender_ask: Vector) -> Result<Amount, Vec<u8>> {
        let sender_itp_minted = sender_bid.data[ORDER_REALIZED_OFFSET];
        let sender_itp_redeem = sender_ask.data[ORDER_REMAIN_OFFSET];
        let sender_itp_burned = sender_ask.data[ORDER_SPENT_OFFSET];

        let sender_balance = sender_itp_minted
            .checked_sub(
                sender_itp_redeem
                    .checked_add(sender_itp_burned)
                    .ok_or_else(|| b"MathOverflow (redeem + burned)")?,
            )
            .ok_or_else(|| b"MathUnderflow (minted < (redeem + burned)")?;

        Ok(sender_balance)
    }
}

pub struct Quote;

impl Quote {
    /// Quote base value of given amount of ITP
    /// 
    /// Base value is calculated as: `Value = Price * Quantity`.
    /// 
    /// Note: We don't use Slope in this calculation, as we don't know side, so
    /// this is more of a mid-point based value.
    /// 
    pub fn tell_base_value(quote: Vector, itp_amount: Amount) -> Result<Amount, Vec<u8>> {
        let base_value = quote.data[QUOTE_PRICE_OFFSET]
            .checked_mul(itp_amount)
            .ok_or_else(|| b"MathOverflow")?;

        Ok(base_value)
    }

    /// Quote amount of ITP corresponding to given base value
    /// 
    /// Base value is calculated as: `Value = Price * Quantity`, and so: `Quantity = Value / Price`.
    /// 
    /// Note: We don't use Slope in this calculation, as we don't know side, so
    /// this is more of a mid-point based quantity.
    /// 
    pub fn tell_itp_amount(quote: Vector, base_value: Amount) -> Result<Amount, Vec<u8>> {
        let itp_amount = base_value
            .checked_div(quote.data[QUOTE_PRICE_OFFSET])
            .ok_or_else(|| b"MathDivisionError")?;
        Ok(itp_amount)
    }
}

pub struct Report;

impl Report {
    pub fn executed_quantity(executed: &Vector) -> Amount {
        executed.data[REPORT_EXECUTED]
    }
    
    pub fn remaining_quantity(executed: &Vector) -> Amount {
        executed.data[REPORT_REMAINING]
    }
}
