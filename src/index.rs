use bytes::Bytes;
use data_encoding::BASE64URL;
use derive_more::Display;
use jsonwebkey as jwk;
use jsonwebkey::JsonWebKey;
use num_derive::FromPrimitive;
use openssl::{hash::MessageDigest, pkey::PKey, rsa::Padding, sign};
use std::panic;

#[cfg(any(feature = "solana", feature = "algorand"))]
use ed25519_dalek::Verifier;

#[cfg(any(feature = "ethereum", feature = "erc20"))]
use crate::{EthereumSigner, Verifier as EthVerifier};

use crate::error::BundlrError;

#[derive(FromPrimitive, Display)]
pub enum SignerMap {
    Arweave = 1,
    Ed25519 = 2,
    Secp256k1 = 3,
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
            #[allow(unreachable_patterns)]
            _ => panic!("{} get_config not implemented in SignerMap yet", self),
        }
    }

    pub fn verify(&self, pk: &[u8], message: &[u8], signature: &[u8]) -> Result<bool, BundlrError> {
        match *self {
            SignerMap::Arweave => {
                let jwt_str = format!(
                    "{{\"kty\":\"RSA\",\"e\":\"AQAB\",\"n\":\"{}\"}}",
                    BASE64URL.encode(pk)
                );
                let jwk: jwk::JsonWebKey = jwt_str.parse().unwrap();
                let p = serde_json::to_string(&jwk).unwrap();
                let key: JsonWebKey = p.parse().unwrap();

                let pkey = PKey::public_key_from_der(key.key.to_der().as_slice()).unwrap();
                let mut verifier = sign::Verifier::new(MessageDigest::sha256(), &pkey).unwrap();
                verifier.set_rsa_padding(Padding::PKCS1_PSS).unwrap();
                verifier.update(message).unwrap();
                verifier
                    .verify(signature)
                    .map_err(|_| BundlrError::InvalidSignature)
            }
            #[cfg(any(feature = "solana", feature = "algorand"))]
            SignerMap::Ed25519 => {
                let public_key = ed25519_dalek::PublicKey::from_bytes(&pk).expect(&format!(
                    "ED25519 public keys must be {} bytes long",
                    ed25519_dalek::PUBLIC_KEY_LENGTH
                ));
                let sig = ed25519_dalek::Signature::from_bytes(&signature).expect(&format!(
                    "ED22519 signatures keys must be {} bytes long",
                    ed25519_dalek::SIGNATURE_LENGTH
                ));
                public_key
                    .verify(message, &sig)
                    .map(|_| true)
                    .map_err(|_| BundlrError::InvalidSignature)
            }
            #[cfg(any(feature = "ethereum", feature = "erc20"))]
            SignerMap::Secp256k1 => EthereumSigner::verify(
                Bytes::copy_from_slice(pk),
                Bytes::copy_from_slice(message),
                Bytes::copy_from_slice(signature),
            ),
            #[allow(unreachable_patterns)]
            _ => panic!("{} verify not implemented in SignerMap yet", self),
        }
    }
}
