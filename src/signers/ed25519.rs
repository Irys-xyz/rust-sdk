use std::array::TryFromSliceError;

use crate::error::BundlrError;
use crate::index::SignerMap;
use crate::Signer as SignerTrait;
use crate::Verifier as VerifierTrait;

use bytes::Bytes;
use ed25519_dalek::{Keypair, Signer, Verifier, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH};

pub struct Ed25519Signer {
    keypair: Keypair,
}

//TODO: add validation for secret keys
impl Ed25519Signer {
    pub fn new(keypair: Keypair) -> Ed25519Signer {
        Ed25519Signer { keypair }
    }

    pub fn from_base58(s: &str) -> Result<Self, BundlrError> {
        let k = bs58::decode(s)
            .into_vec()
            .map_err(|err| BundlrError::ParseError(err.to_string()))?;
        let key: &[u8; 64] = k
            .as_slice()
            .try_into()
            .map_err(|err: TryFromSliceError| BundlrError::ParseError(err.to_string()))?;

        Ok(Self {
            keypair: Keypair::from_bytes(key).map_err(BundlrError::ED25519Error)?,
        })
    }
}

const SIG_TYPE: SignerMap = SignerMap::ED25519;
const SIG_LENGTH: u16 = SIGNATURE_LENGTH as u16;
const PUB_LENGTH: u16 = PUBLIC_KEY_LENGTH as u16;

impl SignerTrait for Ed25519Signer {
    fn sign(&self, message: bytes::Bytes) -> Result<bytes::Bytes, crate::error::BundlrError> {
        Ok(Bytes::copy_from_slice(
            &self.keypair.sign(&message).to_bytes(),
        ))
    }

    fn pub_key(&self) -> bytes::Bytes {
        Bytes::copy_from_slice(&self.keypair.public.to_bytes())
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
    ) -> Result<(), crate::error::BundlrError> {
        let public_key =
            ed25519_dalek::PublicKey::from_bytes(&pk).map_err(BundlrError::ED25519Error)?;
        let sig =
            ed25519_dalek::Signature::from_bytes(&signature).map_err(BundlrError::ED25519Error)?;
        public_key
            .verify(&message, &sig)
            .map_err(|_| BundlrError::InvalidSignature)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Ed25519Signer, Signer, Verifier};
    use bytes::Bytes;
    use ed25519_dalek::Keypair;

    #[test]
    fn should_sign_and_verify() {
        let msg = Bytes::from(b"Message".to_vec());

        let base58_secret_key = "kNykCXNxgePDjFbDWjPNvXQRa8U12Ywc19dFVaQ7tebUj3m7H4sF4KKdJwM7yxxb3rqxchdjezX9Szh8bLcQAjb";
        let signer = Ed25519Signer::from_base58(base58_secret_key).unwrap();
        let sig = signer.sign(msg.clone()).unwrap();
        let pub_key = signer.pub_key();
        println!("{:?}", pub_key.to_vec());
        assert!(Ed25519Signer::verify(pub_key, msg.clone(), sig).is_ok());

        let keypair = Keypair::from_bytes(&[
            237, 158, 92, 107, 132, 192, 1, 57, 8, 20, 213, 108, 29, 227, 37, 8, 3, 105, 196, 244,
            8, 221, 184, 199, 62, 253, 98, 131, 33, 165, 165, 215, 14, 7, 46, 23, 221, 242, 240,
            226, 94, 79, 161, 31, 192, 163, 13, 25, 106, 53, 34, 215, 83, 124, 162, 156, 8, 97,
            194, 180, 213, 179, 33, 68,
        ])
        .unwrap();
        let signer = Ed25519Signer::new(keypair);
        let sig = signer.sign(msg.clone()).unwrap();
        let pub_key = signer.pub_key();

        assert!(Ed25519Signer::verify(pub_key, msg, sig).is_ok());
    }
}
