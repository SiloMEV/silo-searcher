use alloy::primitives::U256;
use crate::uniswapv3pool::abi::IUniswapV3Pool::slot0Return;

#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
pub struct Slot0 {
    pub tick: i32,
    pub fee_protocol: u8,
    pub sqrt_price_x96: U256,
    pub unlocked: bool,
    pub observation_index: u16,
    pub observation_cardinality: u16,
    pub observation_cardinality_next: u16,
}


impl From<slot0Return> for Slot0 {
    fn from(value: slot0Return) -> Self {
        Self {
            tick: value.tick.try_into().unwrap(),
            fee_protocol: value.feeProtocol,
            observation_cardinality: value.observationCardinality,
            observation_cardinality_next: value.observationCardinalityNext,
            sqrt_price_x96: value.sqrtPriceX96.to(),
            unlocked: value.unlocked,
            observation_index: value.observationIndex,
        }
    }
}