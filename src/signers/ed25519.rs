use std::array::TryFromSliceError;

use crate::error::BundlerError;
use crate::index::SignerMap;
use crate::Signer as SignerTrait;
use crate::Verifier as VerifierTrait;

use bytes::Bytes;
use ed25519_dalek::{
    Signer, SigningKey, Verifier, VerifyingKey, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH,
};

pub struct Ed25519Signer {
    signing_key: SigningKey,
}

impl Ed25519Signer {
    pub fn new(signing_key: SigningKey) -> Ed25519Signer {
        Ed25519Signer { signing_key }
    }

    pub fn from_base58(s: &str) -> Result<Self, BundlerError> {
        let k = bs58::decode(s)
            .into_vec()
            .map_err(|err| BundlerError::ParseError(err.to_string()))?;
        let key: &[u8; 64] = k
            .as_slice()
            .try_into()
            .map_err(|err: TryFromSliceError| BundlerError::ParseError(err.to_string()))?;

        Ok(Self {
            signing_key: SigningKey::from_keypair_bytes(key).map_err(BundlerError::ED25519Error)?,
        })
    }
}

const SIG_TYPE: SignerMap = SignerMap::ED25519;
const SIG_LENGTH: u16 = SIGNATURE_LENGTH as u16;
const PUB_LENGTH: u16 = PUBLIC_KEY_LENGTH as u16;

impl SignerTrait for Ed25519Signer {
    fn sign(&self, message: bytes::Bytes) -> Result<bytes::Bytes, crate::error::BundlerError> {
        Ok(Bytes::copy_from_slice(
            &self.signing_key.sign(&message).to_bytes(),
        ))
    }

    fn pub_key(&self) -> bytes::Bytes {
        Bytes::copy_from_slice(&self.signing_key.verifying_key().to_bytes())
    }

    fn sig_type(&self) -> SignerMap {
        SIG_TYPE
    }
    fn get_sig_length(&self) -> u16 {
        SIG_LENGTH
    }
    fn get_pub_length(&self) -> u16 {
        PUB_LENGTH
    }
}

impl VerifierTrait for Ed25519Signer {
    fn verify(
        pk: Bytes,
        message: Bytes,
        signature: Bytes,
    ) -> Result<(), crate::error::BundlerError> {
        let pk_bytes: &[u8; 32] = pk
            .as_ref()
            .try_into()
            .map_err(|_| BundlerError::InvalidKey("public key must be 32 bytes".to_string()))?;
        let public_key = VerifyingKey::from_bytes(pk_bytes).map_err(BundlerError::ED25519Error)?;
        let sig_bytes: &[u8; 64] = signature
            .as_ref()
            .try_into()
            .map_err(|_| BundlerError::InvalidKey("signature must be 64 bytes".to_string()))?;
        let sig = ed25519_dalek::Signature::from_bytes(sig_bytes);
        public_key
            .verify(&message, &sig)
            .map_err(|_| BundlerError::InvalidSignature)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Ed25519Signer, Signer, Verifier};
    use bytes::Bytes;
    use ed25519_dalek::SigningKey;

    #[test]
    fn should_sign_and_verify() {
        let msg = Bytes::from(b"Message".to_vec());

        let base58_secret_key = "kNykCXNxgePDjFbDWjPNvXQRa8U12Ywc19dFVaQ7tebUj3m7H4sF4KKdJwM7yxxb3rqxchdjezX9Szh8bLcQAjb";
        let signer = Ed25519Signer::from_base58(base58_secret_key).unwrap();
        let sig = signer.sign(msg.clone()).unwrap();
        let pub_key = signer.pub_key();
        println!("{:?}", pub_key.to_vec());
        assert!(Ed25519Signer::verify(pub_key, msg.clone(), sig).is_ok());

        let signing_key = SigningKey::from_keypair_bytes(&[
            237, 158, 92, 107, 132, 192, 1, 57, 8, 20, 213, 108, 29, 227, 37, 8, 3, 105, 196, 244,
            8, 221, 184, 199, 62, 253, 98, 131, 33, 165, 165, 215, 14, 7, 46, 23, 221, 242, 240,
            226, 94, 79, 161, 31, 192, 163, 13, 25, 106, 53, 34, 215, 83, 124, 162, 156, 8, 97,
            194, 180, 213, 179, 33, 68,
        ])
        .unwrap();
        let signer = Ed25519Signer::new(signing_key);
        let sig = signer.sign(msg.clone()).unwrap();
        let pub_key = signer.pub_key();

        assert!(Ed25519Signer::verify(pub_key, msg, sig).is_ok());
    }
}
