use crate::{currency::Currency, error::BundlrError, wallet::load_from_file, Signer};

#[cfg(feature = "arweave")]
use crate::ArweaveSigner;
#[cfg(feature = "cosmos")]
use crate::CosmosSigner;
#[cfg(any(feature = "solana", feature = "algorand"))]
use crate::Ed25519Signer;
#[cfg(any(feature = "ethereum", feature = "erc20"))]
use crate::Secp256k1Signer;

#[cfg(feature = "arweave")]
pub mod arweave;
#[cfg(feature = "cosmos")]
pub mod cosmos;
#[cfg(any(feature = "solana", feature = "algorand"))]
pub mod ed25519;
#[cfg(any(feature = "ethereum", feature = "erc20"))]
pub mod secp256k1;

pub mod signer;

pub fn get_signer(c: Currency, w: String) -> Result<Box<dyn Signer>, BundlrError> {
    match c {
        #[cfg(feature = "arweave")]
        Currency::Arweave => {
            let wallet_path = w;
            let jwk = load_from_file(&wallet_path);
            Ok(Box::new(ArweaveSigner::from_jwk(jwk)))
        }

        #[cfg(any(feature = "solana", feature = "algorand"))]
        Currency::Solana => Ok(Box::new(Ed25519Signer::from_base58(&w))),

        #[cfg(any(feature = "ethereum", feature = "erc20"))]
        Currency::Ethereum | Currency::Erc20 => Ok(Box::new(Secp256k1Signer::from_base58(&w))),

        #[cfg(feature = "cosmos")]
        Currency::Cosmos => Ok(Box::new(CosmosSigner::from_base58(&w))),

        #[allow(unreachable_patterns)]
        _ => Err(BundlrError::InvalidCurrency(
            "Could not infer signer from provided currency. Have you enabled its feature?"
                .to_string(),
        )),
    }
}
