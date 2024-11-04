use std::sync::Arc;
use alloy::primitives::{address, U256};
use alloy::providers::ProviderBuilder;
use config::Config;
use crate::uniswapv2pool::UniswapV2Pool;

#[tokio::test]
async fn test_fetch_pool_data() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();

    let cfg = Config::default();
    let rpc_endpoint = cfg.ethereum.unwrap().url.parse().unwrap();
    let provider = Arc::new(ProviderBuilder::new().on_http(rpc_endpoint));
    let addr = address!("C5Be99A02C6857f9Eac67bbCE58DF5572498F40c");

    let _ = UniswapV2Pool::fetch_pool_data(provider.clone(), addr)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_fetch_reserves() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();

    let cfg = Config::default();
    let rpc_endpoint = cfg.ethereum.unwrap().url.parse().unwrap();
    let provider = Arc::new(ProviderBuilder::new().on_http(rpc_endpoint));
    let addr = address!("C5Be99A02C6857f9Eac67bbCE58DF5572498F40c");

    let pool = UniswapV2Pool::new(addr);
    let (_, _) = pool.fetch_reserves(provider).await.unwrap();
}

#[tokio::test]
async fn test_calculate_amount_out() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();

    let cfg = Config::default();
    let rpc_endpoint = cfg.ethereum.unwrap().url.parse().unwrap();
    let provider = Arc::new(ProviderBuilder::new().on_http(rpc_endpoint));
    let addr = address!("C5Be99A02C6857f9Eac67bbCE58DF5572498F40c");

    let pool = UniswapV2Pool::fetch_pool_data(provider.clone(), addr)
        .await
        .unwrap();
    let in_amount = U256::from(U256::from(10).pow(U256::from(18)));
    let out_amount = pool.calculate_out_amount(&pool.token0, &pool.token1, in_amount);
    println!("out_amount {:?}", out_amount);
}
