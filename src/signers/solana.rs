use crate::Verifier;

use super::signer::Signer as SignerTrait;
use bytes::Bytes;
use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signer};

pub struct SolanaSigner {
    key: Keypair,
}

impl SolanaSigner {
    pub fn new(key: Keypair, _public: Vec<u8>) -> SolanaSigner {
        SolanaSigner { key }
    }

    pub fn from_base58(s: &str) -> Self {
        let k = bs58::decode(s).into_vec().expect("Invalid base58 encoding");
        let key: &[u8; 64] = k
            .as_slice()
            .try_into()
            .expect("Couldn't convert base58 key to bytes");
        let sc = SecretKey::from_bytes(&key[..32]).unwrap();
        let pubkey = PublicKey::from_bytes(&key[32..64]).unwrap();

        Self {
            key: Keypair {
                public: pubkey,
                secret: sc,
            },
        }
    }
}

impl SignerTrait for SolanaSigner {
    const SIG_TYPE: u16 = 2;

    const SIG_LENGTH: u16 = 64;

    const PUB_LENGTH: u16 = 32;

    fn sign(&self, message: bytes::Bytes) -> Result<bytes::Bytes, crate::error::BundlrError> {
        Ok(Bytes::copy_from_slice(
            &self.key.sign(&message[..]).to_bytes(),
        ))
    }

    fn pub_key(&self) -> bytes::Bytes {
        Bytes::copy_from_slice(&self.key.public.as_bytes()[..])
    }
}

#[allow(unused)]
impl Verifier for SolanaSigner {
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

    use crate::{Signer, SolanaSigner};

    #[test]
    fn test() {
        let message = &[
            110u8, 123, 209, 66, 178, 255, 153, 11, 45, 235, 189, 244, 42, 9, 152, 192, 181, 183,
            40, 140, 216, 194, 141, 222, 128, 251, 237, 133, 207, 198, 131, 71, 242, 117, 246, 186,
            189, 138, 117, 253, 31, 141, 117, 17, 179, 138, 224, 131,
        ];

        let secret_key = "28PmkjeZqLyfRQogb3FU4E1vJh68dXpbojvS2tcPwezZmVQp8zs8ebGmYg1hNRcjX4DkUALf3SkZtytGWPG3vYhs";
        let signer = SolanaSigner::from_base58(secret_key);
        dbg!(signer.sign(Bytes::from(&message[..])).unwrap().to_vec());
    }
}
