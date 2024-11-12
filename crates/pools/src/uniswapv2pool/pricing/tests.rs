use std::sync::Arc;
use alloy::primitives::{U256};
use alloy::providers::{Provider, ProviderBuilder};
use tracing::info;
use config::Config;
use crate::uniswapv2pool::pricing::{local, router02};
use crate::uniswapv2pool::UniswapV2Pool;

#[tokio::test]
async fn test_calculate_amount_out() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    let cfg = Config::default();
    let net = config::Network::Ethereum;
    let provider = cfg.providers.get(&net).unwrap().clone();
    let addresses = cfg.addresses.get(&net).unwrap().clone();
    let provider = Arc::new(ProviderBuilder::new().on_http(provider.api.parse().unwrap()));

    let block = provider
        .get_block_number()
        .await
        .unwrap();

    let mut pool = UniswapV2Pool::new(
        addresses.uniswap_v2.pools.get("USDC_WETH").unwrap().clone(),
        addresses.uniswap_v2.core.clone()
    );

    pool.data = UniswapV2Pool::fetch_data(
        &pool.metadata,
        provider.clone(),
        block.into(),
    ).await.unwrap();

    pool.state = UniswapV2Pool::fetch_state(
        &pool.metadata,
        &pool.data,
        provider.clone(),
        block.into(),
    ).await.unwrap();

    let amount_in = U256::from(U256::from(10).pow(U256::from(18)));
    let tok_in = pool.data.tok0;
    let amount_out_local = local::calc_amount_out(
        amount_in,
        tok_in,
        pool.data.clone().into(),
        pool.state.clone().into(),
    ).unwrap();
    info!(?amount_out_local);

    let amount_out_router02 = router02::calc_amount_out(
        pool.metadata.core.router02,
        amount_in,
        tok_in,
        pool.data,
        pool.state,
        provider.clone(),
        block.into(),
    )
        .await
        .unwrap();
    info!(?amount_out_router02);

    assert_eq!(amount_out_local, amount_out_router02);
}
