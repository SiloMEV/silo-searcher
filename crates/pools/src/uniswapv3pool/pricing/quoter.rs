use alloy::eips::BlockId;
use alloy::network::Network;
use alloy::primitives::{Address, U160, U256};
use alloy::primitives::aliases::U24;
use alloy::providers::Provider;
use alloy::transports::Transport;
use tracing::instrument;
use crate::uniswapv3pool::pricing::abi::IQuoter;

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
    let quoter = IQuoter::new(quoter_addr, provider.clone());
    let amount_out = quoter
        .quoteExactInputSingle(
            tok_in,
            tok_out,
            U24::from(fee),
            amount_in,
            U160::ZERO,
        )
        .block(block)
        .call()
        .await?
        .amountOut;

    Ok(amount_out)
}

