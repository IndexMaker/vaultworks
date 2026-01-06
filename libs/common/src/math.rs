#[cfg(feature = "amount-sqrt")]
use crate::amount::Amount;

/// Solve quadratic equation:
///  A x^2 + B x - C = 0
///
/// Positive root solution:
///  Q = (sqrt(B^2 + 4 A C) - B) / (2 A)
///
/// NOTE: We are solving quadratic equation with negative term `- C`, which is
/// why in the root solution there is `+` in the part under radical `B^2 + 4 A C`.
/// 
#[cfg(feature = "amount-sqrt")]
pub fn solve_quadratic_bid(a: Amount, b: Amount, negative_c: Amount) -> Option<Amount> {
    let b_squared = b.checked_sq()?;
    let ac = a.checked_mul(negative_c)?;
    let four_ac = ac.checked_mul(Amount::FOUR)?;
    let rad = b_squared.checked_add(four_ac)?;
    let sqrt = rad.checked_sqrt()?;
    let num = sqrt.checked_sub(b)?;
    let den = Amount::TWO.checked_mul(a)?;
    let val = num.checked_div(den)?;
    Some(val)
}

/// Solve quadratic equation:
///  -A x^2 + B x - C = 0
///
/// Positive root solution:
///  Q = (B - sqrt(B^2 - 4 A C)) / (2 A)
///
/// NOTE: We are solving quadratic equation with negative term `- C`, which is
/// why in the root solution there is `+` in the part under radical `B^2 + 4 A C`.
/// 
#[cfg(feature = "amount-sqrt")]
pub fn solve_quadratic_ask(a: Amount, b: Amount, negative_c: Amount) -> Option<Amount> {
    let b_squared = b.checked_sq()?;
    let ac = a.checked_mul(negative_c)?;
    let four_ac = ac.checked_mul(Amount::FOUR)?;
    let rad = b_squared.checked_sub(four_ac)?;
    let sqrt = rad.checked_sqrt()?;
    let num = b.checked_sub(sqrt)?;
    let den = Amount::TWO.checked_mul(a)?;
    let val = num.checked_div(den)?;
    Some(val)
}