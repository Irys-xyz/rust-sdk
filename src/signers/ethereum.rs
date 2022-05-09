use crate::{Signer, Verifier};
use bytes::Bytes;
use secp256k1::{
    constants::{COMPACT_SIGNATURE_SIZE, PUBLIC_KEY_SIZE},
    ecdsa::Signature,
    Message, PublicKey, Secp256k1, SecretKey,
};

pub struct EthereumSigner {
    sec_key: SecretKey,
    pub_key: PublicKey,
}

impl EthereumSigner {
    pub fn new(sec_key: SecretKey) -> EthereumSigner {
        let secp = Secp256k1::new();
        let pub_key = PublicKey::from_secret_key(&secp, &sec_key);
        EthereumSigner { sec_key, pub_key }
    }

    pub fn from_base58(s: &str) -> Self {
        let k = bs58::decode(s).into_vec().expect("Invalid base58 encoding");
        let key: &[u8; 64] = k
            .as_slice()
            .try_into()
            .expect("Couldn't convert base58 key to bytes");

        let sec_key = SecretKey::from_slice(&key[..32]).expect("32 bytes, within curve order");

        Self::new(sec_key)
    }
}

impl Signer for EthereumSigner {
    const SIG_TYPE: u16 = 3;
    const SIG_LENGTH: u16 = COMPACT_SIGNATURE_SIZE as u16;
    const PUB_LENGTH: u16 = PUBLIC_KEY_SIZE as u16;

    fn pub_key(&self) -> bytes::Bytes {
        Bytes::copy_from_slice(&self.pub_key.serialize())
    }

    fn sign(&self, message: bytes::Bytes) -> Result<bytes::Bytes, crate::error::BundlrError> {
        let secp = Secp256k1::new();
        let msg = Message::from_slice(&message).unwrap();
        let sig = secp.sign_ecdsa(&msg, &self.sec_key);

        Ok(Bytes::copy_from_slice(&sig.serialize_compact()))
    }
}

impl Verifier for EthereumSigner {
    fn verify(
        public_key: Bytes,
        message: Bytes,
        signature: Bytes,
    ) -> Result<bool, crate::error::BundlrError> {
        let secp = Secp256k1::new();
        let pub_key = PublicKey::from_slice(&public_key.to_vec())
            .expect("public keys must be 33 or 65 bytes, serialized according to SEC 2");
        let msg = Message::from_slice(&message.to_vec())
            .expect("messages must be 32 bytes and are expected to be hashes");
        let sig = Signature::from_compact(&signature.to_vec())
            .expect("compact signatures are 64 bytes; DER signatures are 68-72 bytes");

        match secp.verify_ecdsa(&msg, &sig, &pub_key) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use secp256k1::SecretKey;

    use crate::{EthereumSigner, Signer, Verifier};

    #[test]
    fn should_create_signer() {
        let secret_key = SecretKey::from_slice(&[0xcd; 32]).expect("");
        EthereumSigner::new(secret_key);

        let base58_secret_key = "28PmkjeZqLyfRQogb3FU4E1vJh68dXpbojvS2tcPwezZmVQp8zs8ebGmYg1hNRcjX4DkUALf3SkZtytGWPG3vYhs";
        EthereumSigner::from_base58(base58_secret_key);
    }

    #[test]
    fn should_sign_and_verify() {
        let msg_bytes = &[
            0xaa, 0xdf, 0x7d, 0xe7, 0x82, 0x03, 0x4f, 0xbe, 0x3d, 0x3d, 0xb2, 0xcb, 0x13, 0xc0,
            0xcd, 0x91, 0xbf, 0x41, 0xcb, 0x08, 0xfa, 0xc7, 0xbd, 0x61, 0xd5, 0x44, 0x53, 0xcf,
            0x6e, 0x82, 0xb4, 0x50,
        ];
        let msg = Bytes::from(&msg_bytes[..]);
        let secret_key = SecretKey::from_slice(&[0xcd; 32]).expect("");
        let signer = EthereumSigner::new(secret_key);

        let sig = signer.sign(msg.clone()).unwrap();
        let pub_key = signer.pub_key();

        assert!(EthereumSigner::verify(pub_key, msg, sig).unwrap());
    }
}
