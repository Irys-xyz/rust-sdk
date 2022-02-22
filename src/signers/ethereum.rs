use crate::Verifier as VerifierTrait;
use bytes::Bytes;
use super::signer::Signer as SignerTrait;
use secp256k1::{Secp256k1, Message, SecretKey, PublicKey};

pub struct EthereumSigner {
    sec_key: SecretKey,
    pub_key: PublicKey
}

#[allow(unused)]
impl EthereumSigner {
    pub fn new(sec_key: SecretKey, pub_key: PublicKey) -> EthereumSigner {
        EthereumSigner { sec_key, pub_key }
    }

    pub fn from_base58(s: &str) -> Self {
        let k = bs58::decode(s).into_vec().expect("Invalid base58 encoding");
        let key: &[u8; 64] = k
            .as_slice()
            .try_into()
            .expect("Couldn't convert base58 key to bytes");

        let secp = Secp256k1::new();
        let sec_key = SecretKey::from_slice(&key[..32])
            .expect("32 bytes, within curve order");
        let pub_key = PublicKey::from_secret_key(&secp, &sec_key);

        Self {
            sec_key,
            pub_key
        }
    }
}

impl SignerTrait for EthereumSigner {
    const SIG_TYPE: u16 = 3;
    const SIG_LENGTH: u16 = 64;
    const PUB_LENGTH: u16 = 33;

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

#[allow(unused)]
impl VerifierTrait for EthereumSigner {
    fn verify(
        pk: Bytes,
        message: Bytes,
        signature: Bytes,
    ) -> Result<bool, crate::error::BundlrError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use crate::{Signer, EthereumSigner};

    #[test]
    fn test_message_sign() {
        let msg = &[
            110u8, 123, 209, 66, 178, 255, 153, 11, 45, 235, 189, 244, 42, 9, 152, 192, 181, 183,
            40, 140, 216, 194, 141, 222, 128, 251, 237, 133, 207, 198, 131, 71, 242, 117, 246, 186,
            189, 138, 117, 253, 31, 141, 117, 17, 179, 138, 224, 131,
        ];

        let signer = EthereumSigner::from_base58("key");
        dbg!(signer.sign(Bytes::from(&msg[..])).unwrap().to_vec());
    }
}