use std::fmt::{Display, Formatter};
use alloy::primitives::{Address, U256};
use serde::{Deserialize, Serialize};

pub trait Pool: Sync + Send {
    fn get_class(&self) -> PoolClass {
        PoolClass::Unknown
    }

    fn get_protocol(&self) -> PoolProtocol {
        PoolProtocol::Unknown
    }

    fn get_address(&self) -> Address;

    fn get_fee(&self) -> u32 { 0 }

    fn get_tokens(&self) -> Vec<Address> {
        Vec::new()
    }

    fn calc_amount_out(
        &self,
        amount_in: U256,
        tok_in: Address,
    ) -> eyre::Result<U256>;

    fn can_flash_swap(&self) -> bool { false }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PoolProtocol {
    Unknown,
    UniswapV2,
    UniswapV2Like,
    UniswapV3,
    UniswapV3Like,
}

impl Default for PoolProtocol {
    fn default() -> Self {
        PoolProtocol::Unknown
    }
}

impl Display for PoolProtocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let protocol_name = match self {
            Self::Unknown => "Unknown",
            Self::UniswapV2 => "UniswapV2",
            Self::UniswapV2Like => "UniswapV2Like",
            Self::UniswapV3 => "UniswapV3",
            Self::UniswapV3Like => "UniswapV3Like",
        };
        write!(f, "{}", protocol_name)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum PoolClass {
    Unknown,
    UniswapV2,
    UniswapV3,
}

