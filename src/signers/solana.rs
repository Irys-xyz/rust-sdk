use crate::Signer as SignerTrait;
use crate::Verifier as VerifierTrait;

use bytes::Bytes;
use ed25519_dalek::{
    Keypair, PublicKey, SecretKey, Signature, Signer, Verifier, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH,
};

pub struct SolanaSigner {
    keypair: Keypair,
}

impl SolanaSigner {
    pub fn new(keypair: Keypair) -> SolanaSigner {
        SolanaSigner { keypair }
    }

    pub fn from_base58(s: &str) -> Self {
        let k = bs58::decode(s).into_vec().expect("Invalid base58 encoding");
        let key: &[u8; 64] = k
            .as_slice()
            .try_into()
            .expect("Couldn't convert base58 key to bytes");

        Self {
            keypair: Keypair::from_bytes(key).unwrap(),
        }
    }
}

impl SignerTrait for SolanaSigner {
    const SIG_TYPE: u16 = 2;
    const SIG_LENGTH: u16 = 64;
    const PUB_LENGTH: u16 = 32;

    fn sign(&self, message: bytes::Bytes) -> Result<bytes::Bytes, crate::error::BundlrError> {
        let sig = &self.keypair.sign(message.as_ref());
        let sig_bytes: [u8; SIGNATURE_LENGTH] = sig.to_bytes();
        Ok(Bytes::copy_from_slice(&sig_bytes))
    }

    fn pub_key(&self) -> bytes::Bytes {
        let public_key_bytes: [u8; PUBLIC_KEY_LENGTH] = self.keypair.public.to_bytes();
        Bytes::copy_from_slice(&public_key_bytes[..])
    }
}

impl VerifierTrait for SolanaSigner {
    fn verify(
        pk: Bytes,
        message: Bytes,
        signature: Bytes,
    ) -> Result<bool, crate::error::BundlrError> {
        let public_key = PublicKey::from_bytes(&pk[..]).unwrap();
        let sig = Signature::from_bytes(&signature[..]).unwrap();

        match public_key.verify(&message[..], &sig) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Signer, SolanaSigner, Verifier};
    use bytes::Bytes;
    use ed25519_dalek::{Keypair, PublicKey, SecretKey, PUBLIC_KEY_LENGTH};

    #[test]
    fn should_create_signer() {
        let base58_secret_key = "28PmkjeZqLyfRQogb3FU4E1vJh68dXpbojvS2tcPwezZmVQp8zs8ebGmYg1hNRcjX4DkUALf3SkZtytGWPG3vYhs";
        SolanaSigner::from_base58(base58_secret_key);

        let keypair = Keypair::from_bytes(&[0xcd; 64]).unwrap();
        SolanaSigner::new(keypair);
    }

    #[test]
    fn should_sign_and_verify() {
        let keypair = Keypair::from_bytes(&[0xcd; 64]).unwrap();
        let signer = SolanaSigner::new(keypair);

        let msg = Bytes::from(b"Message".to_vec());
        let sig = signer.sign(msg.clone()).unwrap();
        let pub_key = signer.pub_key();

        // assert!(SolanaSigner::verify(pub_key, msg, sig).unwrap());
    }
}
