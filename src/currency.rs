use core::fmt;
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};

#[cfg(feature = "build-binary")]
use clap::ValueEnum;

#[derive(FromPrimitive, Debug, Copy, Clone, Hash, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "build-binary", derive(ValueEnum))]
pub enum Currency {
    Arweave = 1,
    Solana = 2,
    Ethereum = 3,
    Erc20 = 4,
    Cosmos = 5,
}

impl fmt::Display for Currency {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", format!("{:?}", self).to_lowercase())
    }
}

/*
impl Currency {
    pub fn needs_fee(&self) -> bool {
        todo!();
    }
    pub async fn get_fee(&self, amount: &BigUint, to: &str) -> BigUint {
        todo!();
    }
    pub async fn create_tx(&self, amount: &BigUint, to: &str, fee: &BigUint) -> BundlrTx {
        todo!();
    }
}
*/
