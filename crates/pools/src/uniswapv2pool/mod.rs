mod abi;
mod pricing;

use alloy::eips::{BlockId};
use alloy::{
    network::Network,
    primitives::{Address, U256},
    providers::Provider,
    transports::Transport,
};
use lazy_static::lazy_static;
use tracing::{instrument};
use types::pool::{Pool, PoolClass, PoolProtocol};
use crate::uniswapv2pool::abi::IUniswapV2Pair;

#[derive(Debug, Clone)]
pub struct PoolMetadata {
    pub pool: Address,
    pub core: config::UniswapV2Core,
    pub protocol: PoolProtocol,
}

#[derive(Debug, Clone, Default)]
pub struct PoolData {
    pub factory: Address,
    pub tok0: Address,
    pub tok1: Address,
    pub fee: u32,
    pub reserves_cell: Option<U256>
}

#[derive(Debug, Clone, Default)]
pub struct PoolState {
    pub reserve0: u128,
    pub reserve1: u128,
}

#[derive(Debug, Clone)]
pub struct UniswapV2Pool {
    pub metadata: PoolMetadata,
    pub data: PoolData,
    pub state: PoolState,
}

impl Pool for UniswapV2Pool {
    fn get_class(&self) -> PoolClass {
        PoolClass::UniswapV2
    }

    fn get_protocol(&self) -> PoolProtocol {
        self.metadata.protocol
    }

    fn get_address(&self) -> Address {
        self.metadata.pool
    }

    fn get_fee(&self) -> u32 {
        self.data.fee
    }

    fn get_tokens(&self) -> Vec<Address> {
        vec![
            self.data.tok0.clone(),
            self.data.tok1.clone()
        ]
    }

    fn calc_amount_out(&self, amount_in: U256, tok_in: Address) -> eyre::Result<U256> {
        pricing::local::calc_amount_out(
            amount_in,
            tok_in,
            self.data.clone().into(),
            self.state.clone().into()
        )
    }
}

#[allow(dead_code)]
impl UniswapV2Pool {
    pub fn new(pool: Address, core: config::UniswapV2Core) -> Self {
        UniswapV2Pool {
            metadata: PoolMetadata {
                pool,
                core,
                protocol: PoolProtocol::UniswapV2Like,
            },
            data: Default::default(),
            state: Default::default(),
        }
    }

    #[instrument(skip_all, level = "debug", ret)]
    pub async fn sync<T: Transport + Clone, N: Network, P: Provider<T, N> + Send + Sync + Clone + 'static>(
        &mut self,
        provider: P,
        block: BlockId
    ) -> eyre::Result<()> {
        self.data = UniswapV2Pool::fetch_data(
            &self.metadata,
            provider.clone(),
            block.into(),
        ).await?;

        self.state = UniswapV2Pool::fetch_state(
            &self.metadata,
            &self.data,
            provider.clone(),
            block.into(),
        ).await?;

        Ok(())
    }

        #[instrument(skip_all, level = "debug", ret)]
    pub async fn fetch_data<T: Transport + Clone, N: Network, P: Provider<T, N> + Send + Sync + Clone + 'static>(
        metadata: &PoolMetadata,
        provider: P,
        block: BlockId,
    ) -> eyre::Result<PoolData> {
        let uni2_pool = IUniswapV2Pair::IUniswapV2PairInstance::new(metadata.pool, provider.clone());

        let tok0: Address = uni2_pool.token0().block(block).call().await?._0;
        let tok1: Address = uni2_pool.token1().block(block).call().await?._0;
        let factory: Address = uni2_pool.factory().block(block).call().await?._0;
        let reserves = uni2_pool.getReserves().block(block).call().await?;

        let storage_reserves_cell =
            provider.get_storage_at(metadata.pool, U256::from(8)).block_id(block).await.unwrap();
        let storage_reserves = storage_to_reserves(storage_reserves_cell);

        let reserves_cell: Option<U256> =
            if storage_reserves.0 == U256::from(reserves.reserve0) && storage_reserves.1 == U256::from(reserves.reserve1) {
                Some(U256::from(8))
            } else {
                None
            };

        Ok(PoolData {
            factory,
            tok0,
            tok1,
            fee: 9970,
            reserves_cell,
        })
    }

    #[instrument(skip(provider), level = "debug", ret)]
    pub async fn fetch_state<T: Transport + Clone, N: Network, P: Provider<T, N> + Send + Sync + Clone + 'static>(
        metadata: &PoolMetadata,
        data: &PoolData,
        provider: P,
        block: BlockId,
    ) -> eyre::Result<PoolState> {
        let (reserve0, reserve1) = match data.reserves_cell {
            Some(cell) => {
                let storage_value = provider.get_storage_at(metadata.pool, cell).block_id(block).await.unwrap();
                let (reserve0, reserve01) = storage_to_reserves(storage_value);
                (reserve0.to::<u128>(), reserve01.to::<u128>())
            }
            None => {
                let uni2_pool = IUniswapV2Pair::IUniswapV2PairInstance::new(metadata.pool, provider.clone());
                let reserves = uni2_pool.getReserves().block(block).call().await?.clone();
                (reserves.reserve0.to::<u128>(), reserves.reserve1.to::<u128>())
            }
        };
        Ok(PoolState{ reserve0, reserve1 })
    }

    fn get_tokens(&self) -> Vec<Address> {
        vec![
            self.data.tok0,
            self.data.tok1
        ]
    }
}

lazy_static! {
    static ref U112_MASK: U256 = (U256::from(1) << 112) - U256::from(1);
}

fn storage_to_reserves(value: U256) -> (U256, U256) {
    ((value >> 0) & *U112_MASK, (value >> (112)) & *U112_MASK)
}