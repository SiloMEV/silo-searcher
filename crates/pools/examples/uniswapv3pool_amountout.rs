use std::sync::Arc;
use alloy::primitives::U256;
use alloy::providers::{Provider, ProviderBuilder};
use tracing::info;
use config::Config;
use pools::uniswapv3pool::UniswapV3Pool;
use types::pool::Pool;

#[tokio::main]
async fn main() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    let cfg = Config::default();
    info!(?cfg);

    let cfg = Config::default();
    let net = config::Network::Ethereum;
    let provider = cfg.providers.get(&net).unwrap().clone();
    let addresses = cfg.addresses.get(&net).unwrap().clone();
    let provider = Arc::new(ProviderBuilder::new().on_http(provider.api.parse().unwrap()));
    let pool_key = "USDC_WETH".to_string();
    let pool_addr = addresses.uniswap_v3.pools.get(&pool_key).unwrap().clone();
    info!("pool: {:?} {:?} {:?}", net, pool_key, pool_addr);

    let block = provider
        .get_block_number()
        .await
        .unwrap();
    info!("block: {:?}", block);

    let mut pool = UniswapV3Pool::new(
        pool_addr,
        addresses.uniswap_v3.periphery.clone()
    );
    pool.sync(provider, block.into()).await.unwrap();
    info!("pool synced");

    let amount_in = U256::from(U256::from(10).pow(U256::from(18)));
    let tok_in = pool.data.tok0;
    let tok_out = pool.data.tok1;
    let amount_out = pool.calc_amount_out(
        amount_in,
        tok_in,
    ).unwrap();
    info!(?amount_in);
    info!(?amount_out);
    info!(?tok_in);
    info!(?tok_out);
}
