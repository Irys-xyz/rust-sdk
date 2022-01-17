use std::borrow::Borrow;

use bytes::Bytes;
use jsonwebkey::JsonWebKey;
use openssl::{sign, hash::MessageDigest, pkey::{PKey, Private}, rsa::Padding};
use serde::Serialize;
use crate::error::BundlrError;
use data_encoding::BASE64URL;

use super::signer::{Signer, Verifier};

#[derive(Serialize)]
struct JWK {
    n: String
}

pub struct ArweaveSigner { 
    priv_key: PKey<Private>
}

impl ArweaveSigner {
    fn new(jwk: JWK) -> ArweaveSigner {
        let n = BASE64URL.decode(jwk.n.as_bytes()).unwrap();
        let s = serde_json::to_string(&jwk).unwrap();
        let key: JsonWebKey = s.parse().unwrap();
        let pem = key.key.to_pem();
        let priv_key  = PKey::private_key_from_pem(pem.as_bytes()).unwrap();

        Self {
            priv_key
        }
    }
}

impl Signer for ArweaveSigner {
    const SIG_TYPE: u16 = 1;
    const SIG_LENGTH: u16 = 512;
    const PUB_LENGTH: u16 = 512;
    fn sign(&self, message: Bytes) -> Result<Bytes, BundlrError> {
        let mut signer = sign::Signer::new(MessageDigest::sha256(), &self.priv_key).unwrap();
        signer.set_rsa_padding(Padding::PKCS1_PSS).unwrap();
        if let Err(_) = signer.update(&message) {
            return Err(BundlrError::NoBytesLeft);
        };

        let mut buf = vec![0;256];
        if let Err(_) = signer.sign(buf.as_mut_slice()) {
            return Err(BundlrError::NoBytesLeft);
        };
        
        return Ok(message.into());
    }

    fn pub_key(&self) -> Bytes {
        self.priv_key.raw_public_key().unwrap().into()
    }
}

impl Verifier for ArweaveSigner {
    fn verify(pk: Bytes, message: Bytes, signature: Bytes) -> Result<bool, BundlrError> {
        let pub_key = PKey::public_key_from_der(&pk).unwrap();
        let mut verifier = sign::Verifier::new(MessageDigest::sha256(), &pub_key).unwrap();
        if let Err(_) = verifier.update(&message) {
            return Err(BundlrError::NoBytesLeft);
        };
        
        verifier.verify(&signature).map_err(|_| BundlrError::NoBytesLeft)
    }

}