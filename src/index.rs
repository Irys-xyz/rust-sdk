use bytes::Bytes;
use derive_more::Display;
use num_derive::FromPrimitive;
use std::panic;

use crate::Verifier;

#[cfg(feature = "arweave")]
use crate::ArweaveSigner;

#[cfg(any(feature = "solana", feature = "algorand"))]
use crate::Ed25519Signer;

#[cfg(any(feature = "ethereum", feature = "erc20"))]
use crate::Secp256k1Signer;

#[cfg(feature = "cosmos")]
use crate::CosmosSigner;

use crate::error::BundlrError;

#[derive(FromPrimitive, Display, PartialEq, Debug, Clone)]
pub enum SignerMap {
    None = -1,
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

impl From<u16> for SignerMap {
    fn from(t: u16) -> Self {
        match t {
            1 => SignerMap::Arweave,
            2 => SignerMap::Ed25519,
            3 => SignerMap::Secp256k1,
            4 => SignerMap::Cosmos,
            _ => panic!("Invalid signer map"),
        }
    }
}

impl SignerMap {
    pub fn as_u16(&self) -> u16 {
        match self {
            SignerMap::Arweave => 1,
            SignerMap::Ed25519 => 2,
            SignerMap::Secp256k1 => 3,
            SignerMap::Cosmos => 4,
            _ => panic!("Invalid signer map"),
        }
    }

    pub fn get_config(&self) -> Config {
        match *self {
            #[cfg(feature = "arweave")]
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
            #[cfg(feature = "arweave")]
            SignerMap::Arweave => ArweaveSigner::verify(
                Bytes::copy_from_slice(pk),
                Bytes::copy_from_slice(message),
                Bytes::copy_from_slice(signature),
            ),
            #[cfg(any(feature = "solana", feature = "algorand"))]
            SignerMap::Ed25519 => Ed25519Signer::verify(
                Bytes::copy_from_slice(pk),
                Bytes::copy_from_slice(message),
                Bytes::copy_from_slice(signature),
            ),
            #[cfg(any(feature = "ethereum", feature = "erc20"))]
            SignerMap::Secp256k1 => Secp256k1Signer::verify(
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
