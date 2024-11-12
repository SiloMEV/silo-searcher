#[cfg(test)]
mod tests;
pub mod local;
pub mod router02;
pub mod abi;

use alloy::primitives::{U256, Address};

/// Ephemeral pool data.
#[derive(Debug, Clone, Default)]
pub struct PoolState {
    pub reserve0: U256,
    pub reserve1: U256,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct PoolData {
    pub fee: U256,
    pub tok0: Address,
    pub tok1: Address,
}

impl From<super::PoolData> for PoolData {
    fn from(val: super::PoolData) -> Self {
        Self {
            fee: U256::from(val.fee),
            tok0: val.tok0,
            tok1: val.tok1,
        }
    }
}

impl From<super::PoolState> for PoolState {
    fn from(val: super::PoolState) -> Self {
        Self {
            reserve0: U256::from(val.reserve0),
            reserve1: U256::from(val.reserve1),
        }
    }
}
