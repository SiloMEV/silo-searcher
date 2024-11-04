#[cfg(test)]
mod tests;

use alloy::eips::BlockNumberOrTag;
use alloy::{
    network::Network,
    primitives::{Address, U256},
    providers::Provider,
    sol,
    transports::Transport,
};
use eyre::{eyre, ErrReport, Result};
use lazy_static::lazy_static;
use tracing::{debug, instrument};
use types::pool::PoolProtocol;

lazy_static! {
    static ref U112_MASK: U256 = (U256::from(1) << 112) - U256::from(1);
}

sol! {
    #[derive(Debug, PartialEq, Eq)]
    #[sol(rpc)]
    contract IUniswapV2Pair {
        event Sync(uint112 reserve0, uint112 reserve1);
        function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
        function token0() external view returns (address);
        function token1() external view returns (address);
        function swap(uint256 amount0Out, uint256 amount1Out, address to, bytes calldata data);
        function factory() external view returns (address);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UniswapV2Pool {
    address: Address,
    token0: Address,
    token1: Address,
    factory: Address,
    reserves_cell: Option<U256>,
    reserves0: U256,
    reserves1: U256,
    protocol: PoolProtocol,
    fee: U256,
}

impl UniswapV2Pool {
    pub fn new(address: Address) -> UniswapV2Pool {
        UniswapV2Pool {
            address,
            token0: Address::ZERO,
            token1: Address::ZERO,
            factory: Address::ZERO,
            reserves_cell: None,
            reserves0: U256::ZERO,
            reserves1: U256::ZERO,
            protocol: PoolProtocol::UniswapV2Like,
            fee: U256::from(9970),
        }
    }

    #[instrument(skip_all, level = "debug", ret)]
    pub async fn fetch_pool_data<T: Transport + Clone, N: Network, P: Provider<T, N> + Send + Sync + Clone + 'static>(
        provider: P,
        address: Address,
    ) -> Result<Self> {
        let uni2_pool = IUniswapV2Pair::IUniswapV2PairInstance::new(address, provider.clone());

        let token0: Address = uni2_pool.token0().call().await?._0;
        let token1: Address = uni2_pool.token1().call().await?._0;
        let factory: Address = uni2_pool.factory().call().await?._0;
        let reserves = uni2_pool.getReserves().call().await?.clone();

        let storage_reserves_cell =
            provider.get_storage_at(address, U256::from(8)).block_id(BlockNumberOrTag::Latest.into()).await.unwrap();
        let storage_reserves = Self::storage_to_reserves(storage_reserves_cell);

        let reserves_cell: Option<U256> =
            if storage_reserves.0 == U256::from(reserves.reserve0) && storage_reserves.1 == U256::from(reserves.reserve1) {
                Some(U256::from(8))
            } else {
                debug!("{storage_reserves:?} {reserves:?}");
                None
            };

        let ret = UniswapV2Pool {
            address,
            token0,
            token1,
            factory,
            reserves_cell,
            reserves0: U256::from(reserves.reserve0),
            reserves1: U256::from(reserves.reserve1),
            protocol: PoolProtocol::UniswapV2Like,
            fee: U256::from(9970),
        };
        Ok(ret)
    }

    #[instrument(skip_all, level = "debug", ret)]
    pub async fn fetch_reserves<T: Transport + Clone, N: Network, P: Provider<T, N> + Send + Sync + Clone + 'static>(
        &self,
        provider: P,
    ) -> Result<(U256, U256)> {
        let (reserve_0, reserve_1) = match self.reserves_cell {
            Some(cell) => {
                let storage_value = provider.get_storage_at(self.address, cell).block_id(BlockNumberOrTag::Latest.into()).await.unwrap();
                Self::storage_to_reserves(storage_value)
            }
            None => {
                let uni2_pool = IUniswapV2Pair::IUniswapV2PairInstance::new(self.address, provider.clone());
                let call_return = uni2_pool.getReserves().call().await?.clone();
                (U256::from(call_return.reserve0), U256::from(call_return.reserve1))
            }
        };
        Ok((reserve_0, reserve_1))
    }

    fn calculate_out_amount(
        &self,
        token_address_from: &Address,
        token_address_to: &Address,
        in_amount: U256,
    ) -> Result<(U256, u64), ErrReport> {
        let (reserves_0, reserves_1) = (self.reserves0, self.reserves1);

        let (reserve_in, reserve_out) = match token_address_from < token_address_to {
            true => (reserves_0, reserves_1),
            false => (reserves_1, reserves_0),
        };

        let amount_in_with_fee = in_amount.checked_mul(self.fee).ok_or(eyre!("AMOUNT_IN_WITH_FEE_OVERFLOW"))?;
        let numerator = amount_in_with_fee.checked_mul(reserve_out).ok_or(eyre!("NUMERATOR_OVERFLOW"))?;
        let denominator = reserve_in.checked_mul(U256::from(10000)).ok_or(eyre!("DENOMINATOR_OVERFLOW"))?;
        let denominator = denominator.checked_add(amount_in_with_fee).ok_or(eyre!("DENOMINATOR_OVERFLOW_FEE"))?;

        let out_amount = numerator.checked_div(denominator).ok_or(eyre!("CANNOT_CALCULATE_ZERO_RESERVE"))?;
        if out_amount > reserve_out {
            Err(eyre!("RESERVE_EXCEEDED"))
        } else if out_amount.is_zero() {
            Err(eyre!("OUT_AMOUNT_IS_ZERO"))
        } else {
            Ok((out_amount, 100_000))
        }
    }

    fn storage_to_reserves(value: U256) -> (U256, U256) {
        ((value >> 0) & *U112_MASK, (value >> (112)) & *U112_MASK)
    }
}
