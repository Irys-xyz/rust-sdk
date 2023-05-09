use crate::error::BundlrError;
use crate::Signer as SignerTrait;
use crate::Verifier as VerifierTrait;
use crate::{index::SignerMap, Ed25519Signer};

use bytes::Bytes;
use ed25519_dalek::{Keypair, Verifier, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH};
use num::Integer;

pub struct AptosSigner {
    signer: Ed25519Signer,
}

impl AptosSigner {
    pub fn new(keypair: Keypair) -> Self {
        Self {
            signer: Ed25519Signer::new(keypair),
        }
    }

    pub fn from_base58(s: &str) -> Result<Self, BundlrError> {
        Ok(Self {
            signer: Ed25519Signer::from_base58(s)?,
        })
    }
}

const SIG_TYPE: SignerMap = SignerMap::InjectedAptos;
const SIG_LENGTH: u16 = SIGNATURE_LENGTH as u16;
const PUB_LENGTH: u16 = PUBLIC_KEY_LENGTH as u16;

impl SignerTrait for AptosSigner {
    fn sign(&self, message: bytes::Bytes) -> Result<bytes::Bytes, crate::error::BundlrError> {
        let aptos_message =
            Bytes::copy_from_slice(&[b"APTOS\nmessage: ".as_ref(), &message[..]].concat());
        let nonce = Bytes::from(b"\nnonce: bundlr".to_vec());
        let full_msg = Bytes::from([aptos_message, nonce].concat());
        self.signer.sign(full_msg)
    }

    fn pub_key(&self) -> bytes::Bytes {
        self.signer.pub_key()
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

impl VerifierTrait for AptosSigner {
    fn verify(
        pk: Bytes,
        message: Bytes,
        signature: Bytes,
    ) -> Result<(), crate::error::BundlrError> {
        let public_key =
            ed25519_dalek::PublicKey::from_bytes(&pk).map_err(BundlrError::ED25519Error)?;
        let sig =
            ed25519_dalek::Signature::from_bytes(&signature).map_err(BundlrError::ED25519Error)?;
        let aptos_message =
            Bytes::copy_from_slice(&[b"APTOS\nmessage: ".as_ref(), &message[..]].concat());
        let nonce = Bytes::from(b"\nnonce: bundlr".to_vec());
        let full_msg = Bytes::from([aptos_message, nonce].concat());

        public_key
            .verify(&full_msg, &sig)
            .map_err(|_err| BundlrError::InvalidSignature)
    }
}

const SIG_TYPE_M: SignerMap = SignerMap::MultiAptos;
const SIG_LENGTH_M: u16 = (SIGNATURE_LENGTH * 32 + 4) as u16; // max 32 64 byte signatures, +4 for 32-bit bitmap
const PUB_LENGTH_M: u16 = (PUBLIC_KEY_LENGTH * 32 + 1) as u16; // max 64 32 byte keys, +1 for 8-bit threshold value

pub struct MultiAptosSigner {
    signer: Ed25519Signer,
}

impl MultiAptosSigner {
    pub fn collect_signatures(
        &self,
        _eamessage: bytes::Bytes,
    ) -> Result<(Vec<bytes::Bytes>, Vec<u64>), crate::error::BundlrError> {
        //TODO: implement
        todo!()
    }
}

impl MultiAptosSigner {
    pub fn new(keypair: Keypair) -> Self {
        Self {
            signer: Ed25519Signer::new(keypair),
        }
    }

    pub fn from_base58(s: &str) -> Result<Self, BundlrError> {
        Ok(Self {
            signer: Ed25519Signer::from_base58(s)?,
        })
    }
}

impl SignerTrait for MultiAptosSigner {
    fn sign(&self, message: bytes::Bytes) -> Result<bytes::Bytes, crate::error::BundlrError> {
        //TODO: implement
        let (_signatures, _bitmap) = self.collect_signatures(message)?;
        todo!()
    }

    fn pub_key(&self) -> bytes::Bytes {
        self.signer.pub_key()
    }

    fn sig_type(&self) -> SignerMap {
        SIG_TYPE_M
    }
    fn get_sig_length(&self) -> u16 {
        SIG_LENGTH_M
    }
    fn get_pub_length(&self) -> u16 {
        PUB_LENGTH_M
    }
}

impl VerifierTrait for MultiAptosSigner {
    fn verify(
        pk: Bytes,
        message: Bytes,
        signature: Bytes,
    ) -> Result<(), crate::error::BundlrError> {
        let sig_len = SIG_LENGTH_M;
        let bitmap_pos = sig_len - 4;
        let signatures = signature.slice(0..(bitmap_pos as usize));
        let encode_bitmap = signature.slice((bitmap_pos as usize)..signature.len());

        let mut one_false = false;
        for i in 0..32 {
            let bucket = i.div_floor(&8);
            let bucket_pos = i - bucket * 8;
            let sig_included = (encode_bitmap[bucket] & (128 >> bucket_pos)) != 0;

            if sig_included {
                let signature = signatures.slice((i * 64)..((i + 1) * 64));
                let pub_key_slc = pk.slice((i * 32)..((i + 1) * 32));
                let public_key = ed25519_dalek::PublicKey::from_bytes(&pub_key_slc)
                    .map_err(BundlrError::ED25519Error)?;
                let sig = ed25519_dalek::Signature::from_bytes(&signature)
                    .map_err(BundlrError::ED25519Error)?;
                match public_key.verify(&message, &sig) {
                    Ok(()) => (),
                    Err(_err) => one_false = false,
                }
            }
        }

        if one_false {
            Err(BundlrError::InvalidSignature)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{AptosSigner, Signer, Verifier};
    use bytes::Bytes;
    use ed25519_dalek::Keypair;

    #[test]
    fn should_sign_and_verify() {
        let msg = Bytes::from(b"Message".to_vec());

        let base58_secret_key = "kNykCXNxgePDjFbDWjPNvXQRa8U12Ywc19dFVaQ7tebUj3m7H4sF4KKdJwM7yxxb3rqxchdjezX9Szh8bLcQAjb";
        let signer = AptosSigner::from_base58(base58_secret_key).unwrap();
        let sig = signer.sign(msg.clone()).unwrap();
        let pub_key = signer.pub_key();
        println!("{:?}", pub_key.to_vec());
        assert!(AptosSigner::verify(pub_key, msg.clone(), sig).is_ok());

        let keypair = Keypair::from_bytes(&[
            237, 158, 92, 107, 132, 192, 1, 57, 8, 20, 213, 108, 29, 227, 37, 8, 3, 105, 196, 244,
            8, 221, 184, 199, 62, 253, 98, 131, 33, 165, 165, 215, 14, 7, 46, 23, 221, 242, 240,
            226, 94, 79, 161, 31, 192, 163, 13, 25, 106, 53, 34, 215, 83, 124, 162, 156, 8, 97,
            194, 180, 213, 179, 33, 68,
        ])
        .unwrap();
        let signer = AptosSigner::new(keypair);
        let sig = signer.sign(msg.clone()).unwrap();
        let pub_key = signer.pub_key();

        assert!(AptosSigner::verify(pub_key, msg, sig).is_ok());
    }

    #[test]
    fn should_sign_and_verify_multisig() {
        //TODO: implement
    }
}
