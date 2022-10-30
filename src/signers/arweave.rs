use std::{path::PathBuf, str::FromStr};

use crate::{error::BundlrError, index::SignerMap, Verifier};
use arweave_rs::ArweaveSigner as SdkSigner;
use bytes::Bytes;

use super::Signer;

pub struct ArweaveSigner {
    sdk: SdkSigner,
}

impl Default for ArweaveSigner {
    fn default() -> Self {
        let path = PathBuf::from_str("wallet.json").expect("Could not open wallet.json");
        Self::from_keypair_path(path).expect("Could not create Arweave Signer")
    }
}

#[allow(unused)]
impl ArweaveSigner {
    pub fn from_keypair_path(keypair_path: PathBuf) -> Result<Self, BundlrError> {
        let sdk = SdkSigner::from_keypair_path(keypair_path).expect("Invalid path");
        Ok(Self { sdk })
    }
}

const SIG_TYPE: SignerMap = SignerMap::Arweave;
const SIG_LENGTH: u16 = 512;
const PUB_LENGTH: u16 = 512;

impl Signer for ArweaveSigner {
    fn sign(&self, message: Bytes) -> Result<Bytes, BundlrError> {
        Ok(Bytes::copy_from_slice(&self.sdk.sign(&message).0))
    }

    fn pub_key(&self) -> Bytes {
        Bytes::copy_from_slice(&self.sdk.get_public_key().0)
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

impl Verifier for ArweaveSigner {
    fn verify(pk: Bytes, message: Bytes, signature: Bytes) -> Result<bool, BundlrError> {
        SdkSigner::verify(&pk, &message, &signature)
            .map(|_| true)
            .map_err(|_| BundlrError::InvalidSignature)
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use crate::{ArweaveSigner, Signer, Verifier};
    use bytes::Bytes;

    #[test]
    fn should_sign_and_verify() {
        let msg = Bytes::copy_from_slice(b"Hello, Bundlr!");
        let path = PathBuf::from_str("res/test_wallet.json").unwrap();
        let signer = ArweaveSigner::from_keypair_path(path).unwrap();

        let sig = signer.sign(msg.clone()).unwrap();
        let pub_key = signer.pub_key();

        assert!(ArweaveSigner::verify(pub_key, msg.clone(), sig).is_ok());
    }
}
