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

#[cfg(feature = "aptos")]
use crate::AptosSigner;

#[cfg(feature = "aptos")]
use crate::MultiAptosSigner;

use crate::error::BundlrError;
use crate::signers::typed_ethereum::TypedEthereumSigner;

#[derive(FromPrimitive, Display, PartialEq, Eq, Debug, Clone)]
pub enum SignerMap {
    None = -1,
    Arweave = 1,
    ED25519 = 2,
    Ethereum = 3,
    Solana = 4,
    InjectedAptos = 5,
    MultiAptos = 6,
    TypedEthereum = 7,
    Cosmos, //TODO: assign constant
}

pub struct Config {
    pub sig_length: usize,
    pub pub_length: usize,
    pub sig_name: String,
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
            2 => SignerMap::ED25519,
            3 => SignerMap::Ethereum,
            4 => SignerMap::Solana,
            5 => SignerMap::InjectedAptos,
            6 => SignerMap::MultiAptos,
            7 => SignerMap::TypedEthereum,
            _ => SignerMap::None,
        }
    }
}

impl SignerMap {
    pub fn as_u16(&self) -> u16 {
        match self {
            SignerMap::Arweave => 1,
            SignerMap::ED25519 => 2,
            SignerMap::Ethereum => 3,
            SignerMap::Solana => 4,
            SignerMap::InjectedAptos => 5,
            SignerMap::MultiAptos => 6,
            SignerMap::TypedEthereum => 7,
            _ => u16::MAX,
        }
    }

    pub fn get_config(&self) -> Config {
        match *self {
            #[cfg(feature = "arweave")]
            SignerMap::Arweave => Config {
                sig_length: 512,
                pub_length: 512,
                sig_name: "arweave".to_owned(),
            },
            #[cfg(feature = "algorand")]
            SignerMap::ED25519 => Config {
                sig_length: ed25519_dalek::SIGNATURE_LENGTH,
                pub_length: ed25519_dalek::PUBLIC_KEY_LENGTH,
                sig_name: "ed25519".to_owned(),
            },
            #[cfg(any(feature = "ethereum", feature = "erc20"))]
            SignerMap::Ethereum => Config {
                sig_length: secp256k1::constants::COMPACT_SIGNATURE_SIZE + 1,
                pub_length: secp256k1::constants::UNCOMPRESSED_PUBLIC_KEY_SIZE,
                sig_name: "ethereum".to_owned(),
            },
            #[cfg(feature = "solana")]
            SignerMap::Solana => Config {
                sig_length: ed25519_dalek::SIGNATURE_LENGTH,
                pub_length: ed25519_dalek::PUBLIC_KEY_LENGTH,
                sig_name: "solana".to_owned(),
            },
            #[cfg(feature = "aptos")]
            SignerMap::InjectedAptos => Config {
                sig_length: ed25519_dalek::SIGNATURE_LENGTH,
                pub_length: ed25519_dalek::PUBLIC_KEY_LENGTH,
                sig_name: "injectedAptos".to_owned(),
            },
            #[cfg(feature = "aptos")]
            SignerMap::MultiAptos => Config {
                sig_length: ed25519_dalek::SIGNATURE_LENGTH * 32 + 4, // max 32 64 byte signatures, +4 for 32-bit bitmap
                pub_length: ed25519_dalek::PUBLIC_KEY_LENGTH * 32 + 1, // max 64 32 byte keys, +1 for 8-bit threshold value
                sig_name: "multiAptos".to_owned(),
            },
            #[cfg(feature = "cosmos")]
            SignerMap::Cosmos => Config {
                sig_length: secp256k1::constants::COMPACT_SIGNATURE_SIZE,
                pub_length: secp256k1::constants::PUBLIC_KEY_SIZE,
                sig_name: "cosmos".to_owned(),
            },
            #[cfg(any(feature = "ethereum", feature = "erc20"))]
            SignerMap::TypedEthereum => Config {
                sig_length: secp256k1::constants::COMPACT_SIGNATURE_SIZE + 1,
                pub_length: 42,
                sig_name: "typedEthereum".to_owned(),
            },
            #[allow(unreachable_patterns)]
            _ => panic!("{:?} get_config has no", self),
        }
    }

    pub fn verify(&self, pk: &[u8], message: &[u8], signature: &[u8]) -> Result<(), BundlrError> {
        match *self {
            #[cfg(feature = "arweave")]
            SignerMap::Arweave => ArweaveSigner::verify(
                Bytes::copy_from_slice(pk),
                Bytes::copy_from_slice(message),
                Bytes::copy_from_slice(signature),
            ),
            #[cfg(feature = "algorand")]
            SignerMap::ED25519 => Ed25519Signer::verify(
                Bytes::copy_from_slice(pk),
                Bytes::copy_from_slice(message),
                Bytes::copy_from_slice(signature),
            ),
            #[cfg(any(feature = "ethereum", feature = "erc20"))]
            SignerMap::Ethereum => Secp256k1Signer::verify(
                Bytes::copy_from_slice(pk),
                Bytes::copy_from_slice(message),
                Bytes::copy_from_slice(signature),
            ),
            #[cfg(feature = "solana")]
            SignerMap::Solana => Ed25519Signer::verify(
                Bytes::copy_from_slice(pk),
                Bytes::copy_from_slice(message),
                Bytes::copy_from_slice(signature),
            ),
            #[cfg(feature = "aptos")]
            SignerMap::InjectedAptos => AptosSigner::verify(
                Bytes::copy_from_slice(pk),
                Bytes::copy_from_slice(message),
                Bytes::copy_from_slice(signature),
            ),
            #[cfg(feature = "aptos")]
            SignerMap::MultiAptos => MultiAptosSigner::verify(
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
            #[cfg(any(feature = "ethereum", feature = "erc20"))]
            SignerMap::TypedEthereum => TypedEthereumSigner::verify(
                Bytes::copy_from_slice(pk),
                Bytes::copy_from_slice(message),
                Bytes::copy_from_slice(signature),
            ),
            #[allow(unreachable_patterns)]
            _ => panic!("{:?} verify not implemented in SignerMap yet", self),
        }
    }
}
