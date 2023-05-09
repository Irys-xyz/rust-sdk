use std::path::PathBuf;

use crate::{error::BundlrError, index::SignerMap, Verifier};
use arweave_rs::ArweaveSigner as SdkSigner;
use bytes::Bytes;

use super::Signer;

pub struct ArweaveSigner {
    sdk: SdkSigner,
}

#[allow(unused)]
impl ArweaveSigner {
    pub fn from_keypair_path(keypair_path: PathBuf) -> Result<Self, BundlrError> {
        let sdk =
            SdkSigner::from_keypair_path(keypair_path).map_err(BundlrError::ArweaveSdkError)?;
        let pub_key = sdk.get_public_key().0;
        if pub_key.len() as u16 == PUB_LENGTH {
            Ok(Self { sdk })
        } else {
            Err(BundlrError::InvalidKey(format!(
                "Public key length should be of {}",
                PUB_LENGTH
            )))
        }
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
    fn verify(pk: Bytes, message: Bytes, signature: Bytes) -> Result<(), BundlrError> {
        SdkSigner::verify(&pk, &message, &signature).map_err(|err| match err {
            arweave_rs::error::Error::InvalidSignature => BundlrError::InvalidSignature,
            _ => BundlrError::ArweaveSdkError(err),
        })
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

        assert!(ArweaveSigner::verify(pub_key, msg, sig).is_ok());
    }
}
