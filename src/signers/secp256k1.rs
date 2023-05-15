use std::array::TryFromSliceError;

use crate::{error::BundlrError, index::SignerMap, Signer, Verifier};
use bytes::Bytes;
use secp256k1::{
    constants::{COMPACT_SIGNATURE_SIZE, UNCOMPRESSED_PUBLIC_KEY_SIZE},
    Message, PublicKey, Secp256k1, SecretKey,
};
use web3::{
    signing::{keccak256, recover},
    types::{Address, H256},
};

pub struct Secp256k1Signer {
    sec_key: SecretKey,
    pub_key: PublicKey,
}

impl Secp256k1Signer {
    pub fn new(sec_key: SecretKey) -> Secp256k1Signer {
        let secp = Secp256k1::new();
        let pub_key = PublicKey::from_secret_key(&secp, &sec_key);
        Secp256k1Signer { sec_key, pub_key }
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

        Ok(Self::new(sec_key))
    }

    pub fn eth_hash_message(msg: &[u8]) -> [u8; 32] {
        let data = &[
            b"\x19Ethereum Signed Message:\n",
            msg.len().to_string().as_bytes(),
            msg,
        ]
        .concat();
        keccak256(data)
    }
}

const SIG_TYPE: SignerMap = SignerMap::Ethereum;
const SIG_LENGTH: u16 = (COMPACT_SIGNATURE_SIZE + 1) as u16;
const PUB_LENGTH: u16 = UNCOMPRESSED_PUBLIC_KEY_SIZE as u16;

impl Signer for Secp256k1Signer {
    fn pub_key(&self) -> bytes::Bytes {
        Bytes::copy_from_slice(&self.pub_key.serialize_uncompressed())
    }

    fn sign(&self, message: bytes::Bytes) -> Result<bytes::Bytes, crate::error::BundlrError> {
        let msg = Message::from_slice(&Secp256k1Signer::eth_hash_message(&message[..]))
            .map_err(BundlrError::Secp256k1Error)?;
        let (recovery_id, signature) = secp256k1::Secp256k1::signing_only()
            .sign_ecdsa_recoverable(&msg, &self.sec_key)
            .serialize_compact();

        let standard_v = recovery_id.to_i32() as u8;
        let r = H256::from_slice(&signature[..32]);
        let s = H256::from_slice(&signature[32..]);
        let v: u8 = standard_v + 27;
        let data = &[r.as_bytes(), s.as_bytes(), &[v]].concat();

        Ok(Bytes::copy_from_slice(data))
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

impl Verifier for Secp256k1Signer {
    fn verify(
        public_key: Bytes,
        message: Bytes,
        signature: Bytes,
    ) -> Result<(), crate::error::BundlrError> {
        let msg = Secp256k1Signer::eth_hash_message(&message);

        let recovery_address = recover(&msg, &signature[0..64], signature[64] as i32 - 27)
            .map_err(BundlrError::RecoveryError)?;

        let pubkey = PublicKey::from_slice(&public_key)
            .map_err(BundlrError::Secp256k1Error)?
            .serialize_uncompressed();
        assert_eq!(pubkey[0], 0x04);
        let pubkey_hash = keccak256(&public_key[1..]);
        let address = Address::from_slice(&pubkey_hash[12..]);

        if address.eq(&recovery_address) {
            return Ok(());
        }

        Err(BundlrError::InvalidSignature)
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use secp256k1::SecretKey;

    use crate::{Secp256k1Signer, Signer, Verifier};

    #[test]
    fn should_hash_message_correctly() {
        let expected: [u8; 32] = [
            115, 94, 155, 26, 251, 67, 239, 226, 251, 85, 181, 193, 50, 136, 70, 88, 238, 217, 84,
            244, 92, 5, 82, 24, 227, 189, 141, 69, 122, 231, 149, 229,
        ];
        let hashed_message = Secp256k1Signer::eth_hash_message(b"Hello, Bundlr!");
        assert_eq!(expected, hashed_message);
    }

    #[test]
    fn should_sign_and_verify() {
        let msg = Bytes::from("Hello, Bundlr!");

        let secret_key = SecretKey::from_slice(b"00000000000000000000000000000000").unwrap();
        let signer = Secp256k1Signer::new(secret_key);
        let sig = signer.sign(msg.clone()).unwrap();
        let pub_key = signer.pub_key();
        assert!(Secp256k1Signer::verify(pub_key, msg.clone(), sig).is_ok());

        let base58_secret_key = "28PmkjeZqLyfRQogb3FU4E1vJh68dXpbojvS2tcPwezZmVQp8zs8ebGmYg1hNRcjX4DkUALf3SkZtytGWPG3vYhs";
        let signer = Secp256k1Signer::from_base58(base58_secret_key).unwrap();
        let sig = signer.sign(msg.clone()).unwrap();
        let pub_key = signer.pub_key();
        assert!(Secp256k1Signer::verify(pub_key, msg, sig).is_ok());
    }
}
