use core::fmt;
use num::{BigRational, BigUint, CheckedMul};
use num_derive::FromPrimitive;
use num_traits::One;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[cfg(feature = "build-binary")]
use clap::ValueEnum;

#[derive(FromPrimitive, Debug, Copy, Clone, Hash, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "build-binary", derive(ValueEnum))]
pub enum Currency {
    Arweave,
    Solana,
    Ethereum,
    Erc20,
    Cosmos,
}

impl fmt::Display for Currency {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", format!("{:?}", self).to_lowercase())
    }
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

impl Currency {
    pub fn needs_fee(&self) -> bool {
        todo!();
    }
    pub async fn get_fee(
        &self,
        _amount: &BigUint,
        _to: &str,
        multiplier: Option<BigRational>,
    ) -> BigUint {
        let base_fee: BigUint = One::one(); //TODO: get fee properly
        if multiplier.is_some() {
            let multiplier = multiplier.unwrap();
            let base_fee = BigRational::from_str(&base_fee.to_string())
                .expect("Error converting BigUInt to BigFloat");
            let base_fee = base_fee
                .checked_mul(&multiplier)
                .expect("Error multiplying two BigRational numbers");
            let base_fee = base_fee.ceil();
            BigUint::from_str(&base_fee.to_string()).expect("Error converting BigInt to BigUint")
        } else {
            base_fee.clone()
        }
    }
    pub async fn create_tx(&self, _amount: &BigUint, _to: &str, _fee: &BigUint) -> () {
        todo!();
    }
}
