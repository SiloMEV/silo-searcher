#[cfg(test)]
mod tests;
use eyre::Result;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub ethereum: Option<EthereumConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self { ethereum: Some(EthereumConfig { url: "https://rpc.ankr.com/eth".to_string() }) }
    }
}

impl Config {
    pub fn load_from_file(file_name: String) -> Result<Config> {
        let contents = fs::read_to_string(file_name)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct EthereumConfig {
    pub url: String,
}
