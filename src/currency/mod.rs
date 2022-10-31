#[cfg(feature = "arweave")]
pub mod arweave;
#[cfg(feature = "solana")]
pub mod solana;

#[cfg(feature = "ethereum")]
pub mod ethereum;

use core::fmt;

use bytes::Bytes;
use num_derive::FromPrimitive;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[cfg(feature = "build-binary")]
use clap::ValueEnum;

use crate::{
    error::BundlrError,
    transaction::{Tx, TxStatus},
    Signer,
};

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
    /// Gets the base unit name, such as "winston" for Arweave
    fn get_min_unit_name(&self) -> String;

    /// Gets currency type
    fn get_type(&self) -> CurrencyType;

    /// Returns if the currency needs fee for transacting
    fn needs_fee(&self) -> bool;

    /// Gets transaction based on transaction id
    async fn get_tx(&self, tx_id: String) -> Result<Tx, BundlrError>;

    /// Gets the transaction status, including height, included block's hash and height
    async fn get_tx_status(
        &self,
        tx_id: String,
    ) -> Result<(StatusCode, Option<TxStatus>), BundlrError>;

    /// Gets public key
    fn get_pub_key(&self) -> Bytes;

    /// Gets wallet address, usually a hash from public key
    fn wallet_address(&self) -> String;

    /// Signs a given message
    fn sign_message(&self, message: &[u8]) -> Vec<u8>;

    /// Verifies if public key, message and signature matches
    fn verify(&self, pub_key: &[u8], message: &[u8], signature: &[u8]) -> Result<(), BundlrError>;

    /// Gets signer for more specific operations
    fn get_signer(&self) -> &dyn Signer;

    /// Gets currency Id
    async fn get_id(&self, item: ()) -> String;

    /// Get price of currency in USD
    async fn price(&self) -> String;

    /// Get given currency network's block height
    async fn get_current_height(&self) -> u128;

    /// Get fee for transaction
    async fn get_fee(&self, amount: u64, to: &str, multiplier: f64) -> u64;

    /// Creates a new transaction
    async fn create_tx(&self, amount: u64, to: &str, fee: u64) -> Tx;

    /// Send a signed transaction
    async fn send_tx(&self, data: Tx) -> Result<TxResponse, BundlrError>;
}
