use alloy::primitives::{Address, U256};
use eyre::eyre;
use tracing::instrument;

#[instrument(level = "debug", ret)]
pub fn calc_amount_out(
    amount_in: U256,
    tok_in: Address,
    data: super::PoolData,
    state: super::PoolState,
) -> eyre::Result<U256> {
    let (reserve_in, reserve_out) = match tok_in == data.tok0 {
        true => (state.reserve0, state.reserve1),
        false => (state.reserve1, state.reserve0),
    };

    let amount_in_with_fee = amount_in.checked_mul(data.fee).ok_or(eyre!("AMOUNT_IN_WITH_FEE_OVERFLOW"))?;
    let numerator = amount_in_with_fee.checked_mul(reserve_out).ok_or(eyre!("NUMERATOR_OVERFLOW"))?;
    let denominator = reserve_in.checked_mul(U256::from(10000)).ok_or(eyre!("DENOMINATOR_OVERFLOW"))?;
    let denominator = denominator.checked_add(amount_in_with_fee).ok_or(eyre!("DENOMINATOR_OVERFLOW_FEE"))?;

    let out_amount = numerator.checked_div(denominator).ok_or(eyre!("CANNOT_CALCULATE_ZERO_RESERVE"))?;
    if out_amount > reserve_out {
        Err(eyre!("RESERVE_EXCEEDED"))
    } else if out_amount.is_zero() {
        Err(eyre!("OUT_AMOUNT_IS_ZERO"))
    } else {
        Ok(out_amount)
    }
}
