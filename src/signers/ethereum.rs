use crate::{Signer, Verifier};
use bytes::Bytes;
use secp256k1::{
    constants::{SCHNORR_SIGNATURE_SIZE, UNCOMPRESSED_PUBLIC_KEY_SIZE},
    Message, PublicKey, Secp256k1, SecretKey,
};
use web3::signing::keccak256;

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

    pub fn eth_hash_message(msg: &[u8]) -> [u8; 32] {
        let data = &[
            b"\x19Ethereum Signed Message:\n",
            msg.len().to_string().as_bytes(),
            msg,
        ]
        .concat();
        keccak256(data)
    }

    pub fn recover_signature(s: &[u8]) -> Result<secp256k1::ecdsa::Signature, secp256k1::Error> {
        let rec_id = secp256k1::ecdsa::RecoveryId::from_i32(s[0] as i32);
        let mut data = [0u8; 64];
        data[0..64].copy_from_slice(&s[1..65]);

        match rec_id {
            Ok(v) => secp256k1::ecdsa::RecoverableSignature::from_compact(&data, v)
                .map(|s| s.to_standard()),
            Err(_) => secp256k1::ecdsa::Signature::from_compact(&data),
        }
    }
}

impl Signer for EthereumSigner {
    const SIG_TYPE: u16 = 3;
    const SIG_LENGTH: u16 = (SCHNORR_SIGNATURE_SIZE + 1) as u16;
    const PUB_LENGTH: u16 = UNCOMPRESSED_PUBLIC_KEY_SIZE as u16;

    fn pub_key(&self) -> bytes::Bytes {
        Bytes::copy_from_slice(&self.pub_key.serialize())
    }

    fn sign(&self, message: bytes::Bytes) -> Result<bytes::Bytes, crate::error::BundlrError> {
        let msg = Message::from_slice(&EthereumSigner::eth_hash_message(&message[..])).unwrap();
        let sig = secp256k1::Secp256k1::signing_only().sign_ecdsa_recoverable(&msg, &self.sec_key);

        let (rec_id, sig_data) = sig.serialize_compact();
        let mut data = [0u8; 65];
        data[0..1].copy_from_slice(&[rec_id.to_i32() as u8]);
        data[1..65].copy_from_slice(&sig_data);

        Ok(Bytes::copy_from_slice(&data))
    }
}

impl Verifier for EthereumSigner {
    fn verify(
        public_key: Bytes,
        message: Bytes,
        signature: Bytes,
    ) -> Result<bool, crate::error::BundlrError> {
        dbg!(&public_key[..], &message[..], &signature[..]);
        let pub_key = PublicKey::from_slice(&public_key.to_vec()).expect(&format!(
            "Public keys must be {} bytes long",
            EthereumSigner::PUB_LENGTH
        ));
        let msg = Message::from_slice(&EthereumSigner::eth_hash_message(&message[..])).unwrap();
        let sig = EthereumSigner::recover_signature(&signature[..]).expect(&format!(
            "Signatures must be {} bytes long",
            EthereumSigner::SIG_LENGTH
        ));

        match secp256k1::Secp256k1::verification_only().verify_ecdsa(&msg, &sig, &pub_key) {
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
        let msg = Bytes::from("Hello, Bundlr!");
        let secret_key = SecretKey::from_slice(&[0xcd; 32]).expect("");
        let signer = EthereumSigner::new(secret_key);

        let sig = signer.sign(msg.clone()).unwrap();
        dbg!(&sig[..]);
        let pub_key = signer.pub_key();

        assert!(EthereumSigner::verify(pub_key, msg, sig).unwrap());
    }
}
