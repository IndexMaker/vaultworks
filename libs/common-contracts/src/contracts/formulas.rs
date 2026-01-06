use alloc::vec::Vec;

use common::{amount::Amount, vector::Vector};

pub const ORDER_REMAIN_OFFSET: usize = 0;
pub const ORDER_SPENT_OFFSET: usize = 1;
pub const ORDER_REALIZED_OFFSET: usize = 2;
pub const ORDER_LAST_OFFSET: usize = 3;

pub const QUOTE_CAPACITY_OFFSET: usize = 0;
pub const QUOTE_PRICE_OFFSET: usize = 1;
pub const QUOTE_SLOPE_OFFSET: usize = 2;
pub const QUOTE_LAST_OFFSET: usize = 3;

pub const REPORT_DELIVERED_OFFSET: usize = 0;
pub const REPORT_RECEIVED_OFFSET: usize = 1;
pub const REPORT_LAST_OFFSET: usize = 2;

pub struct Order {
    pub bid: Vector,
    pub ask: Vector,
}

impl Order {
    pub fn try_from_vec_pair(
        sender_bid_bytes: Vec<u8>,
        sender_ask_bytes: Vec<u8>,
    ) -> Result<Self, Vec<u8>> {
        let this = Self {
            bid: Vector::from_vec(sender_bid_bytes),
            ask: Vector::from_vec(sender_ask_bytes),
        };
        if this.bid.data.len() != ORDER_LAST_OFFSET || this.ask.data.len() != ORDER_LAST_OFFSET {
            Err(b"Invalid data size")?;
        }
        Ok(this)
    }

    pub fn try_from_vec(data: Vec<u8>) -> Result<Self, Vec<u8>> {
        if data.len() != ORDER_LAST_OFFSET * 2 * size_of::<u128>() {
            Err(b"Invalid data size")?;
        }
        let mut bid = Vector::from_vec(data);
        let ask_data = bid.data.drain(ORDER_LAST_OFFSET..);
        let ask = Vector {
            data: Vec::from_iter(ask_data),
        };
        let this = Self { bid, ask };
        Ok(this)
    }

    pub fn to_vec(self) -> Vec<u8> {
        Self::encode_vec_pair(self.bid.to_vec(), self.ask.to_vec())
    }

    pub fn encode_vec_pair(mut bid_bytes: Vec<u8>, ask_bytes: Vec<u8>) -> Vec<u8> {
        bid_bytes.extend(ask_bytes);
        bid_bytes
    }

    pub fn collateral_remaining(&self) -> Amount {
        self.bid.data[ORDER_REMAIN_OFFSET]
    }

    pub fn collateral_spent(&self) -> Amount {
        self.bid.data[ORDER_SPENT_OFFSET]
    }

    pub fn itp_minted(&self) -> Amount {
        self.ask.data[ORDER_REALIZED_OFFSET]
    }

    pub fn itp_locked(&self) -> Amount {
        self.ask.data[ORDER_REMAIN_OFFSET]
    }

    pub fn itp_burned(&self) -> Amount {
        self.ask.data[ORDER_SPENT_OFFSET]
    }

    pub fn collateral_withdrawn(&self) -> Amount {
        self.ask.data[ORDER_REALIZED_OFFSET]
    }

    /// Total amount of ITP in existence
    ///
    /// This includes all ITP that was minted and not burned.
    ///
    /// This is total amount, which includes active balance that can be used
    /// plus any balance locked for redeem, but not redeemed yet.
    ///
    pub fn tell_total(&self) -> Result<Amount, Vec<u8>> {
        let sender_balance = self
            .itp_minted()
            .checked_sub(self.itp_burned())
            .ok_or_else(|| b"MathUnderflow (minted < (redeem + burned)")?;

        Ok(sender_balance)
    }

    /// Available amount of ITP
    ///
    /// This includes all ITP that was minted and neither burned nor pending redeem.
    ///
    /// This is an active balance of an account, that can be used.
    ///
    pub fn tell_available(&self) -> Result<Amount, Vec<u8>> {
        let sender_balance = self
            .itp_minted()
            .checked_sub(
                self.itp_locked()
                    .checked_add(self.itp_burned())
                    .ok_or_else(|| b"MathOverflow (redeem + burned)")?,
            )
            .ok_or_else(|| b"MathUnderflow (minted < (redeem + burned)")?;

        Ok(sender_balance)
    }
}

pub struct Quote {
    pub quote: Vector,
}

impl Quote {
    pub fn try_from_vec(quote_bytes: Vec<u8>) -> Result<Self, Vec<u8>> {
        let this = Self {
            quote: Vector::from_vec(quote_bytes),
        };
        if this.quote.data.len() != QUOTE_LAST_OFFSET {
            Err(b"Invalid data size")?;
        }
        Ok(this)
    }

    pub fn to_vec(self) -> Vec<u8> {
        self.quote.to_vec()
    }

    pub fn capacity(&self) -> Amount {
        self.quote.data[QUOTE_CAPACITY_OFFSET]
    }

    pub fn price(&self) -> Amount {
        self.quote.data[QUOTE_PRICE_OFFSET]
    }

    pub fn slope(&self) -> Amount {
        self.quote.data[QUOTE_SLOPE_OFFSET]
    }

    /// Quote base value of given amount of ITP
    ///
    /// Base value is calculated as: `Value = Price * Quantity`.
    ///
    /// Note: We don't use Slope in this calculation, as we don't know side, so
    /// this is more of a mid-point based value.
    ///
    pub fn tell_base_value(&self, itp_amount: Amount) -> Result<Amount, Vec<u8>> {
        let base_value = self
            .price()
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
    pub fn tell_itp_amount(&self, base_value: Amount) -> Result<Amount, Vec<u8>> {
        let itp_amount = base_value
            .checked_div(self.price())
            .ok_or_else(|| b"MathDivisionError")?;
        Ok(itp_amount)
    }
}

pub struct Report {
    pub report: Vector,
}

impl Report {
    pub fn try_from_vec(repor_bytes: Vec<u8>) -> Result<Self, Vec<u8>> {
        let this = Self {
            report: Vector::from_vec(repor_bytes),
        };
        if this.report.data.len() != REPORT_LAST_OFFSET {
            Err(b"Invalid data size")?;
        }
        Ok(this)
    }

    pub fn to_vec(self) -> Vec<u8> {
        self.report.to_vec()
    }

    pub fn delivered(&self) -> Amount {
        self.report.data[REPORT_DELIVERED_OFFSET]
    }

    pub fn received(&self) -> Amount {
        self.report.data[REPORT_RECEIVED_OFFSET]
    }
}
