use std::collections::HashMap;
use alloy::primitives::{Address, I256, U256};
use eyre::eyre;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uniswap_v3_math::tick_math::{MAX_SQRT_RATIO, MAX_TICK, MIN_SQRT_RATIO, MIN_TICK};
use crate::uniswapv3pool::slot0::Slot0;
use crate::uniswapv3pool::{UniswapV3PoolData, UniswapV3PoolState};

pub const U256_1: U256 = U256::from_limbs([1, 0, 0, 0]);

pub struct CurrentState {
    amount_specified_remaining: I256,
    amount_calculated: I256,
    sqrt_price_x_96: U256,
    tick: i32,
    liquidity: u128,
}

#[derive(Default)]
pub struct StepComputations {
    pub sqrt_price_start_x_96: U256,
    pub tick_next: i32,
    pub initialized: bool,
    pub sqrt_price_next_x96: U256,
    pub amount_in: U256,
    pub amount_out: U256,
    pub fee_amount: U256,
}

#[allow(dead_code)]
pub struct Tick {
    pub liquidity_gross: u128,
    pub liquidity_net: i128,
    pub fee_growth_outside_0_x_128: U256,
    pub fee_growth_outside_1_x_128: U256,
    pub tick_cumulative_outside: U256,
    pub seconds_per_liquidity_outside_x_128: U256,
    pub seconds_outside: u32,
    pub initialized: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TickInfo {
    pub liquidity_gross: u128,
    pub liquidity_net: i128,
}

/// Constant pool data.
#[derive(Debug)]
pub struct PoolData {
    pub tok0: Address,
    pub fee: u32,
    pub tick_spacing: u32,
}

impl From<UniswapV3PoolData> for PoolData {
    fn from(val: UniswapV3PoolData) -> Self {
        Self {
            tok0: val.tok0,
            fee: val.fee,
            tick_spacing: val.tick_spacing,
        }
    }
}

/// Ephemeral pool data.
#[derive(Debug)]
pub struct PoolState {
    pub slot0: Slot0,
    pub liquidity: u128,
    pub tick_bitmap: HashMap<i16, U256>,
    pub ticks: HashMap<i32, TickInfo>,
}

impl From<UniswapV3PoolState> for PoolState {
    fn from(val: UniswapV3PoolState) -> Self {
        Self {
            slot0: val.slot0,
            liquidity: val.liquidity,
            tick_bitmap: val.tick_bitmap,
            ticks: val.ticks,
        }
    }
}

#[instrument(level = "debug", ret)]
pub fn calc_amount_out(
    amount_in: U256,
    tok_in: Address,
    pool_data: PoolData,
    pool_state: PoolState,
) -> eyre::Result<U256> {
    if amount_in.is_zero() {
        return Ok(U256::ZERO);
    }

    let zero_for_one = tok_in == pool_data.tok0;

    // Set sqrt_price_limit_x_96 to the max or min sqrt price in the pool depending on zero_for_one
    let sqrt_price_limit_x_96 = if zero_for_one {
        MIN_SQRT_RATIO + U256_1
    } else {
        MAX_SQRT_RATIO - U256_1
    };

    // Initialize a mutable state struct to hold the dynamic simulated state of the pool
    let mut current_state = CurrentState {
        sqrt_price_x_96: pool_state.slot0.sqrt_price_x96, //Active price on the pool
        amount_calculated: I256::ZERO,    //Amount of token_out that has been calculated
        amount_specified_remaining: I256::from_raw(amount_in), //Amount of token_in that has not been swapped
        tick: pool_state.slot0.tick,                                       //Current i24 tick of the pool
        liquidity: pool_state.liquidity, //Current available liquidity in the tick range
    };

    while current_state.amount_specified_remaining != I256::ZERO
        && current_state.sqrt_price_x_96 != sqrt_price_limit_x_96
    {
        // Initialize a new step struct to hold the dynamic state of the pool at each step
        let mut step = StepComputations {
            // Set the sqrt_price_start_x_96 to the current sqrt_price_x_96
            sqrt_price_start_x_96: current_state.sqrt_price_x_96,
            ..Default::default()
        };

        // Get the next tick from the current tick
        (step.tick_next, step.initialized) =
            uniswap_v3_math::tick_bitmap::next_initialized_tick_within_one_word(
                &pool_state.tick_bitmap,
                current_state.tick,
                pool_data.tick_spacing.try_into().unwrap(),
                zero_for_one,
            )?;

        // ensure that we do not overshoot the min/max tick, as the tick bitmap is not aware of these bounds
        // Note: this could be removed as we are clamping in the batch contract
        step.tick_next = step.tick_next.clamp(MIN_TICK, MAX_TICK);

        // Get the next sqrt price from the input amount
        step.sqrt_price_next_x96 =
            uniswap_v3_math::tick_math::get_sqrt_ratio_at_tick(step.tick_next)?;

        // Target spot price
        let swap_target_sqrt_ratio = if zero_for_one {
            if step.sqrt_price_next_x96 < sqrt_price_limit_x_96 {
                sqrt_price_limit_x_96
            } else {
                step.sqrt_price_next_x96
            }
        } else if step.sqrt_price_next_x96 > sqrt_price_limit_x_96 {
            sqrt_price_limit_x_96
        } else {
            step.sqrt_price_next_x96
        };

        // Compute swap step and update the current state
        (
            current_state.sqrt_price_x_96,
            step.amount_in,
            step.amount_out,
            step.fee_amount,
        ) = uniswap_v3_math::swap_math::compute_swap_step(
            current_state.sqrt_price_x_96,
            swap_target_sqrt_ratio,
            current_state.liquidity,
            current_state.amount_specified_remaining,
            pool_data.fee,
        )?;

        // Decrement the amount remaining to be swapped and amount received from the step
        current_state.amount_specified_remaining = current_state
            .amount_specified_remaining
            .overflowing_sub(I256::from_raw(
                step.amount_in.overflowing_add(step.fee_amount).0,
            ))
            .0;

        current_state.amount_calculated -= I256::from_raw(step.amount_out);

        // If the price moved all the way to the next price, recompute the liquidity change for the next iteration
        if current_state.sqrt_price_x_96 == step.sqrt_price_next_x96 {
            if step.initialized {
                let mut liquidity_net = if let Some(info) = pool_state.ticks.get(&step.tick_next) {
                    info.liquidity_net
                } else {
                    0
                };

                // we are on a tick boundary, and the next tick is initialized, so we must charge a protocol fee
                if zero_for_one {
                    liquidity_net = -liquidity_net;
                }

                current_state.liquidity = if liquidity_net < 0 {
                    if current_state.liquidity < (-liquidity_net as u128) {
                        return Err(eyre!("LIQUIDITY_UNDERFLOW"));
                    } else {
                        current_state.liquidity - (-liquidity_net as u128)
                    }
                } else {
                    current_state.liquidity + (liquidity_net as u128)
                };
            }
            // Increment the current tick
            current_state.tick = if zero_for_one {
                step.tick_next.wrapping_sub(1)
            } else {
                step.tick_next
            }
            // If the current_state sqrt price is not equal to the step sqrt price, then we are not on the same tick.
            // Update the current_state.tick to the tick at the current_state.sqrt_price_x_96
        } else if current_state.sqrt_price_x_96 != step.sqrt_price_start_x_96 {
            current_state.tick = uniswap_v3_math::tick_math::get_tick_at_sqrt_ratio(
                current_state.sqrt_price_x_96,
            )?;
        }
    }

    if current_state.amount_specified_remaining.is_zero() {
        let amount_out = (-current_state.amount_calculated).into_raw();
        tracing::trace!("amount_out : {amount_out}");
        Ok(amount_out)
    } else {
        Err(eyre!("NOT_ENOUGH_LIQUIDITY"))
    }
}

