use crate::error::BundlrError;
use bytes::Bytes;
use data_encoding::BASE64URL;
use jsonwebkey as jwk;
use rand::thread_rng;
use rsa::{
    pkcs8::{DecodePrivateKey, DecodePublicKey},
    PaddingScheme, PublicKey, PublicKeyParts, RsaPrivateKey, RsaPublicKey,
};
use sha2::Digest;

use super::signer::{Signer, Verifier};

pub struct ArweaveSigner {
    priv_key: RsaPrivateKey,
}

#[allow(unused)]
impl ArweaveSigner {
    fn new(priv_key: RsaPrivateKey) -> ArweaveSigner {
        Self { priv_key }
    }

    pub fn from_jwk(jwk: jwk::JsonWebKey) -> ArweaveSigner {
        let pem = jwk.key.to_pem();
        let priv_key = RsaPrivateKey::from_pkcs8_pem(&pem).unwrap();

        Self { priv_key }
    }
}

const SIG_TYPE: u16 = 1;
const SIG_LENGTH: u16 = 512;
const PUB_LENGTH: u16 = 512;

impl Signer for ArweaveSigner {
    fn sign(&self, message: Bytes) -> Result<Bytes, BundlrError> {
        let mut hasher = sha2::Sha256::new();
        hasher.update(&message);
        let hashed = hasher.finalize();

        let rng = thread_rng();
        let padding = PaddingScheme::PSS {
            salt_rng: Box::new(rng),
            digest: Box::new(sha2::Sha256::new()),
            salt_len: None,
        };

        let signature = self
            .priv_key
            .sign(padding, &hashed)
            .map_err(|e| BundlrError::SigningError(e.to_string()))?;

        Ok(signature.into())
    }

    fn pub_key(&self) -> Bytes {
        self.priv_key.to_public_key().n().to_bytes_be().into()
    }

    fn sig_type(&self) -> u16 {
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
        let jwt_str = format!(
            "{{\"kty\":\"RSA\",\"e\":\"AQAB\",\"n\":\"{}\"}}",
            BASE64URL.encode(&pk[..])
        );
        let jwk: jwk::JsonWebKey = jwt_str.parse().unwrap();

        let pub_key = RsaPublicKey::from_public_key_der(jwk.key.to_der().as_slice()).unwrap();
        let mut hasher = sha2::Sha256::new();
        hasher.update(&message);
        let hashed = &hasher.finalize();

        let rng = thread_rng();
        let padding = PaddingScheme::PSS {
            salt_rng: Box::new(rng),
            digest: Box::new(sha2::Sha256::new()),
            salt_len: None,
        };
        pub_key
            .verify(padding, hashed, &signature)
            .map(|_| true)
            .map_err(|_| BundlrError::InvalidSignature)
    }
}

#[cfg(test)]
mod tests {
    use crate::{wallet, ArweaveSigner, Signer, Verifier};
    use bytes::Bytes;
    use jsonwebkey as jwk;

    #[test]
    fn should_sign_and_verify() {
        let msg = Bytes::copy_from_slice(b"Hello, Bundlr!");
        let jwk: jwk::JsonWebKey = wallet::load_from_file("res/test_wallet.json");
        let signer = ArweaveSigner::from_jwk(jwk);

        let sig = signer.sign(msg.clone()).unwrap();
        let pub_key = signer.pub_key();

        assert!(ArweaveSigner::verify(pub_key, msg.clone(), sig).is_ok());
    }
}
