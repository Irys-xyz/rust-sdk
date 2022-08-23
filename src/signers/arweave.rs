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
    use crate::{ArweaveSigner, Signer, Verifier};
    use bytes::Bytes;
    use jsonwebkey as jwk;

    #[test]
    fn should_sign_and_verify() {
        let msg = Bytes::copy_from_slice(b"Hello, Bundlr!");
        let jwk_str = r#"{
            "kty" : "RSA",
            "kid" : "cc34c0a0-bd5a-4a3c-a50d-a2a7db7643df",
            "use" : "sig",
            "n"   : "pjdss8ZaDfEH6K6U7GeW2nxDqR4IP049fk1fK0lndimbMMVBdPv_hSpm8T8EtBDxrUdi1OHZfMhUixGaut-3nQ4GG9nM249oxhCtxqqNvEXrmQRGqczyLxuh-fKn9Fg--hS9UpazHpfVAFnB5aCfXoNhPuI8oByyFKMKaOVgHNqP5NBEqabiLftZD3W_lsFCPGuzr4Vp0YS7zS2hDYScC2oOMu4rGU1LcMZf39p3153Cq7bS2Xh6Y-vw5pwzFYZdjQxDn8x8BG3fJ6j8TGLXQsbKH1218_HcUJRvMwdpbUQG5nvA2GXVqLqdwp054Lzk9_B_f1lVrmOKuHjTNHq48w",
            "e"   : "AQAB",
            "d"   : "ksDmucdMJXkFGZxiomNHnroOZxe8AmDLDGO1vhs-POa5PZM7mtUPonxwjVmthmpbZzla-kg55OFfO7YcXhg-Hm2OWTKwm73_rLh3JavaHjvBqsVKuorX3V3RYkSro6HyYIzFJ1Ek7sLxbjDRcDOj4ievSX0oN9l-JZhaDYlPlci5uJsoqro_YrE0PRRWVhtGynd-_aWgQv1YzkfZuMD-hJtDi1Im2humOWxA4eZrFs9eG-whXcOvaSwO4sSGbS99ecQZHM2TcdXeAs1PvjVgQ_dKnZlGN3lTWoWfQP55Z7Tgt8Nf1q4ZAKd-NlMe-7iqCFfsnFwXjSiaOa2CRGZn-Q",
            "p"   : "4A5nU4ahEww7B65yuzmGeCUUi8ikWzv1C81pSyUKvKzu8CX41hp9J6oRaLGesKImYiuVQK47FhZ--wwfpRwHvSxtNU9qXb8ewo-BvadyO1eVrIk4tNV543QlSe7pQAoJGkxCia5rfznAE3InKF4JvIlchyqs0RQ8wx7lULqwnn0",
            "q"   : "ven83GM6SfrmO-TBHbjTk6JhP_3CMsIvmSdo4KrbQNvp4vHO3w1_0zJ3URkmkYGhz2tgPlfd7v1l2I6QkIh4Bumdj6FyFZEBpxjE4MpfdNVcNINvVj87cLyTRmIcaGxmfylY7QErP8GFA-k4UoH_eQmGKGK44TRzYj5hZYGWIC8",
            "dp"  : "lmmU_AG5SGxBhJqb8wxfNXDPJjf__i92BgJT2Vp4pskBbr5PGoyV0HbfUQVMnw977RONEurkR6O6gxZUeCclGt4kQlGZ-m0_XSWx13v9t9DIbheAtgVJ2mQyVDvK4m7aRYlEceFh0PsX8vYDS5o1txgPwb3oXkPTtrmbAGMUBpE",
            "dq"  : "mxRTU3QDyR2EnCv0Nl0TCF90oliJGAHR9HJmBe__EjuCBbwHfcT8OG3hWOv8vpzokQPRl5cQt3NckzX3fs6xlJN4Ai2Hh2zduKFVQ2p-AF2p6Yfahscjtq-GY9cB85NxLy2IXCC0PF--Sq9LOrTE9QV988SJy_yUrAjcZ5MmECk",
            "qi"  : "ldHXIrEmMZVaNwGzDF9WG8sHj2mOZmQpw9yrjLK9hAsmsNr5LTyqWAqJIYZSwPTYWhY4nu2O0EY9G9uYiqewXfCKw_UngrJt8Xwfq1Zruz0YY869zPN4GiE9-9rzdZB33RBw8kIOquY3MK74FMwCihYx_LiU2YTHkaoJ3ncvtvg"
        }"#;
        let jwk: jwk::JsonWebKey = jwk_str.parse().unwrap();
        let signer = ArweaveSigner::from_jwk(jwk);

        let sig = signer.sign(msg.clone()).unwrap();
        let pub_key = signer.pub_key();

        assert!(ArweaveSigner::verify(pub_key, msg.clone(), sig).is_ok());
    }
}
