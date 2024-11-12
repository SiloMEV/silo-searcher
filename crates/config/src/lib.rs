#[cfg(test)]
mod tests;

use std::collections::HashMap;
use eyre::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use alloy::primitives::Address;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub providers: HashMap<Network, NetworkProvider>,
    pub addresses: HashMap<Network, NetworkAddresses>
}

impl Default for Config {
    fn default() -> Self {
        let workspace_dir = workspace_dir()
            .join("config-default.toml")
            .to_str()
            .unwrap()
            .to_string();

        Config::load_from_file(workspace_dir)
            .context("config-default.toml file missing")
            .unwrap()
    }
}

impl Config {
    pub fn load_from_file(file_name: String) -> Result<Config> {
        let contents = fs::read_to_string(file_name)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Debug, Deserialize)]
pub enum Network {
    Ethereum,
}

#[derive(Hash, Eq, PartialEq, Clone, Debug, Deserialize)]
pub struct NetworkProvider {
    pub api: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NetworkAddresses {
    pub uniswap_v2: UniswapV2,
    pub uniswap_v3: UniswapV3,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UniswapV2 {
    pub pools: HashMap<String, Address>,
    pub core: UniswapV2Core,
    pub periphery: UniswapV2Periphery
}

#[derive(Clone, Debug, Deserialize)]
pub struct UniswapV2Core {
    pub router02: Address,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UniswapV2Periphery {}

#[derive(Clone, Debug, Deserialize)]
pub struct UniswapV3 {
    pub pools: HashMap<String, Address>,
    pub core: UniswapV3Core,
    pub periphery: UniswapV3Periphery
}

#[derive(Clone, Debug, Deserialize)]
pub struct UniswapV3Core {}


#[derive(Clone, Debug, Deserialize)]
pub struct UniswapV3Periphery {
    pub quoter: Address,
    pub quoter_v2: Address,
    pub tick_lens: Address,
}


fn workspace_dir() -> PathBuf {
    let output = std::process::Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .unwrap()
        .stdout;
    let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
    cargo_path.parent().unwrap().to_path_buf()
}
