use crate::uniswapv3pool::pricing::local::PoolData;
use crate::uniswapv3pool::UniswapV3PoolData;

#[cfg(test)]
mod tests;
pub mod abi;
pub mod local;
pub mod quoter;
pub mod quoter2;

impl From<UniswapV3PoolData> for PoolData {
    fn from(val: UniswapV3PoolData) -> Self {
        Self {
            tok0: val.tok0,
            fee: val.fee,
            tick_spacing: val.tick_spacing,
        }
    }
}



