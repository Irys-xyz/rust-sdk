use num_derive::FromPrimitive;
use std::str::FromStr;
use strum_macros::Display;

#[cfg(feature = "build-binary")]
use clap::ValueEnum;

#[derive(FromPrimitive, Debug, Copy, Clone, Display)]
#[cfg_attr(feature = "build-binary", derive(ValueEnum))]
#[strum(serialize_all = "snake_case")]
pub enum Currency {
    Arweave = 1,
    Solana = 2,
    Ethereum = 3,
    Erc20 = 4,
    Cosmos = 5,
}

impl FromStr for Currency {
    type Err = anyhow::Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "arweave" => Ok(Currency::Arweave),
            "solana" => Ok(Currency::Solana),
            "ethereum" => Ok(Currency::Ethereum),
            "erc20" => Ok(Currency::Erc20),
            "cosmos" => Ok(Currency::Cosmos),
            _ => Err(anyhow::Error::msg("Invalid or unsupported currency")),
        }
    }
}
