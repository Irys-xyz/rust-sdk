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
    error::BundlerError,
    transaction::{Tx, TxStatus},
    Signer,
};

#[derive(FromPrimitive, Debug, Copy, Clone, Hash, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "build-binary", derive(ValueEnum))]
pub enum TokenType {
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

impl fmt::Display for TokenType {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", format!("{:?}", self).to_lowercase())
    }
}

impl FromStr for TokenType {
    type Err = anyhow::Error;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "arweave" => Ok(TokenType::Arweave),
            "solana" => Ok(TokenType::Solana),
            "ethereum" => Ok(TokenType::Ethereum),
            "erc20" => Ok(TokenType::Erc20),
            "cosmos" => Ok(TokenType::Cosmos),
            _ => Err(anyhow::Error::msg("Invalid or unsupported currency")),
        }
    }
}

#[async_trait::async_trait]
pub trait Currency {
    /// Gets the base unit name, such as "winston" for Arweave
    fn get_min_unit_name(&self) -> String;

    /// Gets currency type
    fn get_type(&self) -> TokenType;

    /// Returns if the currency needs fee for transacting
    fn needs_fee(&self) -> bool;

    /// Gets transaction based on transaction id
    async fn get_tx(&self, tx_id: String) -> Result<Tx, BundlerError>;

    /// Gets the transaction status, including height, included block's hash and height
    async fn get_tx_status(
        &self,
        tx_id: String,
    ) -> Result<(StatusCode, Option<TxStatus>), BundlerError>;

    /// Gets public key
    fn get_pub_key(&self) -> Result<Bytes, BundlerError>;

    /// Gets wallet address, usually a hash from public key
    fn wallet_address(&self) -> Result<String, BundlerError>;

    /// Signs a given message
    fn sign_message(&self, message: &[u8]) -> Result<Vec<u8>, BundlerError>;

    /// Verifies if public key, message and signature matches
    fn verify(&self, pub_key: &[u8], message: &[u8], signature: &[u8]) -> Result<(), BundlerError>;

    /// Gets signer for more specific operations
    fn get_signer(&self) -> Result<&dyn Signer, BundlerError>;

    /// Gets currency Id
    async fn get_id(&self, item: ()) -> String;

    /// Get price of currency in USD
    async fn price(&self) -> String;

    /// Get given currency network's block height
    async fn get_current_height(&self) -> u128;

    /// Get fee for transaction
    async fn get_fee(&self, amount: u64, to: &str, multiplier: f64) -> Result<u64, BundlerError>;

    /// Creates a new transaction
    async fn create_tx(&self, amount: u64, to: &str, fee: u64) -> Result<Tx, BundlerError>;

    /// Send a signed transaction
    async fn send_tx(&self, data: Tx) -> Result<TxResponse, BundlerError>;
}
