use alloy::eips::BlockId;
use alloy::network::Network;
use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use alloy::transports::Transport;
use tracing::instrument;
use crate::uniswapv2pool::{PoolData, PoolState};
use crate::uniswapv2pool::pricing::abi::IUniswapV2Router02;

#[instrument(skip(provider), level = "debug", ret)]
pub async fn calc_amount_out<T: Transport + Clone, N: Network, P: Provider<T, N> + Send + Sync + Clone + 'static>(
    router02: Address,
    amount_in: U256,
    tok_in: Address,
    data: PoolData,
    state: PoolState,
    provider: P,
    block: BlockId,
) -> eyre::Result<U256> {
    let (reserve_in, reserve_out) = match tok_in == data.tok0 {
        true => (state.reserve0, state.reserve1),
        false => (state.reserve1, state.reserve0),
    };

    let router02 = IUniswapV2Router02::new(router02, provider.clone());
    let amount_out = router02
        .getAmountOut(
            amount_in,
            U256::from(reserve_in),
            U256::from(reserve_out),
        )
        .block(block)
        .call()
        .await?
        .amountOut;

    Ok(amount_out)
}

