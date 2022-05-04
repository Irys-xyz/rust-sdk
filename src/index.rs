use data_encoding::BASE64URL;
use derive_more::Display;
use jsonwebkey::JsonWebKey;
use num_derive::FromPrimitive;
use openssl::{hash::MessageDigest, pkey::PKey, rsa::Padding, sign};
use serde::Serialize;
use std::panic;

#[cfg(feature = "ethereum")]
use secp256k1::Secp256k1;

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

#[derive(Serialize)]
pub struct JWK<'a> {
    pub kty: &'a str,
    pub e: &'a str,
    pub n: String,
}

impl SignerMap {
    pub fn get_config(&self) -> Config {
        match *self {
            SignerMap::Arweave => Config {
                sig_length: 512,
                pub_length: 512,
            },
            SignerMap::Ed25519 => Config {
                sig_length: 64,
                pub_length: 32,
            },
            SignerMap::Secp256k1 => Config {
                sig_length: 64,
                pub_length: 32,
            },
            #[allow(unreachable_patterns)]
            _ => panic!("{} get_config not implemented in SignerMap yet", self),
        }
    }

    pub fn verify(&self, pk: &[u8], message: &[u8], signature: &[u8]) -> Result<bool, BundlrError> {
        match *self {
            SignerMap::Arweave => {
                let jwk = JWK {
                    kty: "RSA",
                    e: "AQAB",
                    n: BASE64URL.encode(pk),
                };
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
            SignerMap::Ed25519 => {
                let pkey = PKey::public_key_from_raw_bytes(pk, openssl::pkey::Id::ED25519)
                    .expect("Couldn't create PKey<Public>");
                let mut verifier = sign::Verifier::new(MessageDigest::null(), &pkey).unwrap();
                verifier
                    .verify_oneshot(signature, message)
                    .map_err(|_| BundlrError::InvalidSignature)
            }
            #[cfg(any(feature = "ethereum", feature = "erc20"))]
            SignerMap::Secp256k1 => {
                let verifier = Secp256k1::verification_only();
                let pub_key = secp256k1::PublicKey::from_slice(pk).unwrap();
                let msg = secp256k1::Message::from_slice(message).unwrap();
                let sig = secp256k1::ecdsa::Signature::from_compact(signature).unwrap();

                verifier
                    .verify_ecdsa(&msg, &sig, &pub_key)
                    .map(|_| true)
                    .map_err(|_| BundlrError::InvalidSignature)
            }
            #[allow(unreachable_patterns)]
            _ => panic!("{} verify not implemented in SignerMap yet", self),
        }
    }
}
