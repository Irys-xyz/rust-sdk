use std::array::TryFromSliceError;

use crate::{error::BundlrError, index::SignerMap, Signer, Verifier};
use bytes::Bytes;
use secp256k1::{
    constants::{COMPACT_SIGNATURE_SIZE, PUBLIC_KEY_SIZE},
    Message, PublicKey, Secp256k1, SecretKey,
};
use sha2::Digest;

pub struct CosmosSigner {
    sec_key: SecretKey,
    pub_key: PublicKey,
}

impl CosmosSigner {
    pub fn new(sec_key: SecretKey) -> Result<CosmosSigner, BundlrError> {
        let secp = Secp256k1::new();
        let pub_key = PublicKey::from_secret_key(&secp, &sec_key);
        if pub_key.serialize().len() == PUBLIC_KEY_SIZE {
            Ok(Self { sec_key, pub_key })
        } else {
            Err(BundlrError::InvalidKey(format!(
                "Public key length should be of {}",
                PUB_LENGTH
            )))
        }
    }

    pub fn from_base58(s: &str) -> Result<Self, BundlrError> {
        let k = bs58::decode(s)
            .into_vec()
            .map_err(|err| BundlrError::ParseError(err.to_string()))?;
        let key: &[u8; 64] = k
            .as_slice()
            .try_into()
            .map_err(|err: TryFromSliceError| BundlrError::ParseError(err.to_string()))?;

        let sec_key = SecretKey::from_slice(&key[..32])
            .map_err(|err| BundlrError::ParseError(err.to_string()))?;

        Self::new(sec_key)
    }

    pub fn sha256_digest(msg: &[u8]) -> [u8; 32] {
        let mut hasher = sha2::Sha256::new();
        hasher.update(msg);
        let result = hasher.finalize();
        result.into()
    }
}

const SIG_TYPE: SignerMap = SignerMap::Cosmos;
const SIG_LENGTH: u16 = COMPACT_SIGNATURE_SIZE as u16;
const PUB_LENGTH: u16 = PUBLIC_KEY_SIZE as u16;

impl Signer for CosmosSigner {
    fn pub_key(&self) -> bytes::Bytes {
        let pub_key = &self.pub_key.serialize();
        assert!(pub_key.len() == PUBLIC_KEY_SIZE);
        Bytes::copy_from_slice(pub_key)
    }

    fn sign(&self, message: bytes::Bytes) -> Result<bytes::Bytes, crate::error::BundlrError> {
        let msg = Message::from_slice(&CosmosSigner::sha256_digest(&message[..]))
            .map_err(BundlrError::Secp256k1Error)?;
        let signature = secp256k1::Secp256k1::signing_only()
            .sign_ecdsa(&msg, &self.sec_key)
            .serialize_compact();

        Ok(Bytes::copy_from_slice(&signature))
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

impl Verifier for CosmosSigner {
    fn verify(
        public_key: Bytes,
        message: Bytes,
        signature: Bytes,
    ) -> Result<(), crate::error::BundlrError> {
        let msg = secp256k1::Message::from_slice(&CosmosSigner::sha256_digest(&message))
            .map_err(BundlrError::Secp256k1Error)?;
        let sig = secp256k1::ecdsa::Signature::from_compact(&signature)
            .map_err(BundlrError::Secp256k1Error)?;
        let pk =
            secp256k1::PublicKey::from_slice(&public_key).map_err(BundlrError::Secp256k1Error)?;

        secp256k1::Secp256k1::verification_only()
            .verify_ecdsa(&msg, &sig, &pk)
            .map_err(|_| BundlrError::InvalidSignature)
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use secp256k1::SecretKey;

    use crate::{CosmosSigner, Signer, Verifier};

    #[test]
    fn should_hash_message_correctly() {
        let expected: [u8; 32] = [
            242, 32, 241, 161, 45, 243, 110, 65, 225, 215, 2, 87, 174, 67, 33, 202, 65, 130, 179,
            30, 170, 249, 140, 26, 152, 240, 228, 194, 31, 206, 193, 109,
        ];
        let hashed_message = CosmosSigner::sha256_digest(b"Hello, Bundlr!");
        assert_eq!(expected, hashed_message);
    }

    #[test]
    fn should_sign_and_verify() {
        let msg = Bytes::from("Hello, Bundlr!");

        let secret_key = SecretKey::from_slice(b"00000000000000000000000000000000").unwrap();
        let signer = CosmosSigner::new(secret_key).unwrap();
        let sig = signer.sign(msg.clone()).unwrap();
        let pub_key = signer.pub_key();
        assert!(CosmosSigner::verify(pub_key, msg.clone(), sig).is_ok());

        let base58_secret_key = "28PmkjeZqLyfRQogb3FU4E1vJh68dXpbojvS2tcPwezZmVQp8zs8ebGmYg1hNRcjX4DkUALf3SkZtytGWPG3vYhs";
        let signer = CosmosSigner::from_base58(base58_secret_key).unwrap();
        let sig = signer.sign(msg.clone()).unwrap();
        let pub_key = signer.pub_key();
        assert!(CosmosSigner::verify(pub_key, msg, sig).is_ok());
    }
}
