use std::collections::HashMap;
use alloy::eips::BlockId;
use alloy::network::Network;
use alloy::primitives::{address, Address, U256};
use alloy::providers::Provider;
use alloy::transports::Transport;
use tracing::instrument;
use types::pool::{Pool, PoolClass, PoolProtocol};
use crate::uniswapv3pool::abi::IUniswapV3Pool;
use crate::uniswapv3pool::pricing::abi::ITickLens;
use crate::uniswapv3pool::pricing::local::TickInfo;
use crate::uniswapv3pool::slot0::Slot0;

#[cfg(test)]
mod tests;
mod slot0;
mod abi;
mod pricing;

#[derive(Debug, Clone, Default)]
pub struct UniswapV3PoolMetadata {
    pub addr: Address,
    pub protocol: PoolProtocol,
}

#[derive(Debug, Clone, Default)]
pub struct UniswapV3PoolData {
    pub factory: Address,
    pub tok0: Address,
    pub tok1: Address,
    pub fee: u32,
    pub tick_spacing: u32,
}

#[derive(Debug, Clone, Default)]
pub struct UniswapV3PoolState {
    pub slot0: Slot0,
    pub liquidity: u128,
    pub tick_bitmap: HashMap<i16, U256>,
    pub ticks: HashMap<i32, TickInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct UniswapV3Pool {
    metadata: UniswapV3PoolMetadata,
    data: UniswapV3PoolData,
    state: UniswapV3PoolState,
}

impl Pool for UniswapV3Pool {
    fn get_class(&self) -> PoolClass {
        PoolClass::UniswapV2
    }

    fn get_protocol(&self) -> PoolProtocol {
        self.metadata.protocol
    }

    fn get_address(&self) -> Address {
        self.metadata.addr
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
impl UniswapV3Pool {
    pub fn new(addr: Address) -> Self {
        UniswapV3Pool {
            metadata: UniswapV3PoolMetadata {
                addr,
                protocol: PoolProtocol::UniswapV3Like,
            },
            data: Default::default(),
            state: Default::default(),
        }
    }

    // TODO: make batch request
    #[instrument(skip_all, level = "debug", ret)]
    pub async fn fetch_data<T: Transport + Clone, N: Network, P: Provider<T, N> + Send + Sync + Clone + 'static>(
        metadata: &UniswapV3PoolMetadata,
        provider: P,
        block: BlockId,
    ) -> eyre::Result<UniswapV3PoolData> {
        let v3_pool = IUniswapV3Pool::IUniswapV3PoolInstance::new(metadata.addr, provider.clone());

        let tok0: Address = v3_pool.token0().block(block).call().await?._0;
        let tok1: Address = v3_pool.token1().block(block).call().await?._0;
        let fee: u32 = v3_pool.fee().block(block).call().await?._0.try_into()?;
        let factory: Address = v3_pool.factory().block(block).call().await?._0;

        Ok(UniswapV3PoolData {
            factory,
            tok0,
            tok1,
            fee,
            tick_spacing: UniswapV3Pool::tick_spacing(fee),
        })
    }

    #[allow(dead_code)]
    #[instrument(skip(provider), level = "debug", ret)]
    pub async fn fetch_state<T: Transport + Clone, N: Network, P: Provider<T, N> + Send + Sync + Clone + 'static>(
        metadata: &UniswapV3PoolMetadata,
        data: &UniswapV3PoolData,
        provider: P,
        block: BlockId,
    ) -> eyre::Result<UniswapV3PoolState> {
        let v3_pool = IUniswapV3Pool::IUniswapV3PoolInstance::new(metadata.addr, provider.clone());

        let liquidity: u128 = v3_pool.liquidity().block(block).call().await?._0;
        let slot0: Slot0 = v3_pool.slot0().block(block).call().await?.into();

        let mut tick_bitmap: HashMap<i16, U256> = Default::default();
        let mut ticks: HashMap<i32, TickInfo> = Default::default();

        let tick_lens = ITickLens::new(
            address!("bfd8137f7d1516d3ea5ca83523914859ec47f573"),
            provider.clone(),
        );
        let tick_bitmap_index = UniswapV3Pool::get_tick_bitmap_index(
            slot0.tick,
            data.tick_spacing,
        );
        for i in -4..=3 { // TODO: define the range better
            let next_index = tick_bitmap_index + i;
            let populated_ticks = tick_lens.getPopulatedTicksInWord(
                metadata.addr,
                next_index,
            )
                .call()
                .block(block)
                .await?;

            for populated_tick in populated_ticks.populatedTicks {
                ticks.insert(
                    populated_tick.tick.try_into().unwrap(),
                    TickInfo {
                        liquidity_gross: populated_tick.liquidityGross,
                        liquidity_net: populated_tick.liquidityNet,
                    },
                );
            }

            let tick_bitmap_value: U256 = v3_pool.tickBitmap(next_index).block(block).call().await?._0;
            tick_bitmap.insert(
                next_index,
                tick_bitmap_value,
            );
        }

        Ok(UniswapV3PoolState {
            slot0,
            liquidity,
            tick_bitmap,
            ticks,
        })
    }

    fn get_tokens(&self) -> Vec<Address> {
        vec![
            self.data.tok0,
            self.data.tok1
        ]
    }

    fn get_other_token(&self, tok: Address) -> Address {
        if tok == self.data.tok0 { self.data.tok1 } else { self.data.tok0 }
    }

    pub fn tick_spacing(fee: u32) -> u32 {
        Self::get_price_step(fee)
    }

    pub fn get_price_step(fee: u32) -> u32 {
        match fee {
            10000 => 200,
            3000 => 60,
            500 => 10,
            100 => 1,
            _ => 0,
        }
    }

    pub fn get_tick_bitmap_index(tick: i32, spacing: u32) -> i16 {
        let (word_pos, _bit_pos) = uniswap_v3_math::tick_bitmap::position(tick / (spacing as i32));
        word_pos
    }
}
