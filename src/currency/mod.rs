#[cfg(feature = "arweave")]
pub mod arweave;

use core::fmt;
use num::{BigRational, BigUint};
use num_derive::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[cfg(feature = "build-binary")]
use clap::ValueEnum;

use crate::{error::BundlrError, transaction::{ Tx, TxStatus }, Signer};

#[derive(FromPrimitive, Debug, Copy, Clone, Hash, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "build-binary", derive(ValueEnum))]
pub enum CurrencyType {
    Arweave = 1,
    Solana = 2,
    Ethereum = 3,
    Erc20 = 4,
    Cosmos = 5,
}

#[derive(Deserialize)]
pub struct TxResponse {
    pub tx_id: String,
}

impl fmt::Display for CurrencyType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", format!("{:?}", self).to_lowercase())
    }
}

impl FromStr for CurrencyType {
    type Err = anyhow::Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "arweave" => Ok(CurrencyType::Arweave),
            "solana" => Ok(CurrencyType::Solana),
            "ethereum" => Ok(CurrencyType::Ethereum),
            "erc20" => Ok(CurrencyType::Erc20),
            "cosmos" => Ok(CurrencyType::Cosmos),
            _ => Err(anyhow::Error::msg("Invalid or unsupported currency")),
        }
    }
}

#[async_trait::async_trait]
pub trait Currency {
    fn get_type(&self) -> CurrencyType;
    fn needs_fee(&self) -> bool;
    async fn get_tx(&self, tx_id: String) -> Result<Tx, BundlrError>;
    async fn get_tx_status(&self, tx_id: String) -> TxStatus;
    fn owner_to_address(&self, owner: String) -> String;
    fn get_signer(&self) -> &dyn Signer;
    async fn get_id(&self, item: ()) -> String;
    async fn price(&self) -> String;
    async fn get_current_height(&self) -> u128;
    async fn get_fee(&self, amount: u64, to: &str, multiplier: f64) -> u64;
    async fn create_tx(&self, amount: u64, to: &str, fee: u64) -> Tx;
    async fn send_tx(&self, data: Tx) -> Result<TxResponse, BundlrError>;
}
