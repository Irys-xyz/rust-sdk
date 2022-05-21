use bytes::Bytes;
use derive_more::Display;
use num_derive::FromPrimitive;
use std::panic;

use crate::{ArweaveSigner, Verifier};

#[cfg(any(feature = "solana", feature = "algorand"))]
use crate::{SolanaSigner};

#[cfg(any(feature = "ethereum", feature = "erc20"))]
use crate::{EthereumSigner};

#[cfg(feature = "cosmos")]
use crate::{CosmosSigner};

use crate::error::BundlrError;

#[derive(FromPrimitive, Display)]
pub enum SignerMap {
    Arweave = 1,
    Ed25519 = 2,
    Secp256k1 = 3,
    Cosmos = 4,
}

pub struct Config {
    pub sig_length: usize,
    pub pub_length: usize,
}

#[allow(unused)]
impl Config {
    pub fn total_length(&self) -> u32 {
        self.sig_length as u32 + self.pub_length as u32
    }
}

impl SignerMap {
    pub fn get_config(&self) -> Config {
        match *self {
            SignerMap::Arweave => Config {
                sig_length: 512,
                pub_length: 512,
            },
            #[cfg(any(feature = "solana", feature = "algorand"))]
            SignerMap::Ed25519 => Config {
                sig_length: ed25519_dalek::SIGNATURE_LENGTH,
                pub_length: ed25519_dalek::PUBLIC_KEY_LENGTH,
            },
            #[cfg(any(feature = "ethereum", feature = "erc20"))]
            SignerMap::Secp256k1 => Config {
                sig_length: secp256k1::constants::COMPACT_SIGNATURE_SIZE + 1,
                pub_length: secp256k1::constants::UNCOMPRESSED_PUBLIC_KEY_SIZE,
            },
            #[cfg(feature = "cosmos")]
            SignerMap::Cosmos => Config {
                sig_length: secp256k1::constants::COMPACT_SIGNATURE_SIZE,
                pub_length: secp256k1::constants::PUBLIC_KEY_SIZE,
            },
            #[allow(unreachable_patterns)]
            _ => panic!("{} get_config not implemented in SignerMap yet", self),
        }
    }

    pub fn verify(&self, pk: &[u8], message: &[u8], signature: &[u8]) -> Result<bool, BundlrError> {
        match *self {
            SignerMap::Arweave => ArweaveSigner::verify(
                Bytes::copy_from_slice(pk),
                Bytes::copy_from_slice(message),
                Bytes::copy_from_slice(signature),
            ),
            #[cfg(any(feature = "solana", feature = "algorand"))]
            SignerMap::Ed25519 => SolanaSigner::verify(
                Bytes::copy_from_slice(pk),
                Bytes::copy_from_slice(message),
                Bytes::copy_from_slice(signature),
            ),
            #[cfg(any(feature = "ethereum", feature = "erc20"))]
            SignerMap::Secp256k1 => EthereumSigner::verify(
                Bytes::copy_from_slice(pk),
                Bytes::copy_from_slice(message),
                Bytes::copy_from_slice(signature),
            ),
            #[cfg(feature = "cosmos")]
            SignerMap::Cosmos => CosmosSigner::verify(
                Bytes::copy_from_slice(pk),
                Bytes::copy_from_slice(message),
                Bytes::copy_from_slice(signature),
            ),
            #[allow(unreachable_patterns)]
            _ => panic!("{} verify not implemented in SignerMap yet", self),
        }
    }
}
