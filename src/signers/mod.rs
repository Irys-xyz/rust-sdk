use crate::{error::BundlrError, index::SignerMap};
use bytes::Bytes;

#[cfg(feature = "aptos")]
pub mod aptos;
#[cfg(feature = "arweave")]
pub mod arweave;
#[cfg(feature = "cosmos")]
pub mod cosmos;
#[cfg(any(feature = "solana", feature = "algorand", feature = "aptos"))]
pub mod ed25519;
#[cfg(any(feature = "ethereum", feature = "erc20"))]
pub mod secp256k1;
#[cfg(any(feature = "ethereum", feature = "erc20"))]
pub mod typed_ethereum;

pub trait ToPem {}

pub trait Signer: Send + Sync {
    fn sign(&self, message: Bytes) -> Result<Bytes, BundlrError>;
    fn sig_type(&self) -> SignerMap;
    fn get_sig_length(&self) -> u16;
    fn get_pub_length(&self) -> u16;
    fn pub_key(&self) -> Bytes;
}
