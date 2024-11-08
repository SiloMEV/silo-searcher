use alloy::eips::BlockId;
use alloy::network::Network;
use alloy::primitives::{Address, U160, U256};
use alloy::primitives::aliases::U24;
use alloy::providers::Provider;
use alloy::transports::Transport;
use tracing::instrument;
use crate::uniswapv3pool::pricing::abi::IQuoterV2;

#[instrument(skip(provider), level = "debug", ret)]
pub async fn calc_amount_out<T: Transport + Clone, N: Network, P: Provider<T, N> + Send + Sync + Clone + 'static>(
    quoter_addr: Address,
    amount_in: U256,
    tok_in: Address,
    tok_out: Address,
    fee: u32,
    provider: P,
    block: BlockId,
) -> eyre::Result<U256> {
    let quoter = IQuoterV2::new(quoter_addr, provider.clone());
    let amount_out = quoter
        .quoteExactInputSingle(
            IQuoterV2::QuoteExactInputSingleParams {
                tokenIn: tok_in,
                tokenOut: tok_out,
                amountIn: amount_in,
                fee: U24::from(fee),
                sqrtPriceLimitX96: U160::ZERO,
            }
        )
        .block(block)
        .call()
        .await?
        .amountOut;

    Ok(amount_out)
}

