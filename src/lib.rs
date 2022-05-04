extern crate derive_builder;
#[cfg(feature = "solana")]
extern crate ed25519_dalek;

pub mod deep_hash;
pub mod deep_hash_sync;
pub mod error;
mod index;
mod signers;
pub mod tags;
mod transaction;
// pub mod stream;
mod bundlr;
pub mod verify;

#[cfg(feature = "solana")]
pub use signers::solana::SolanaSigner;
pub use transaction::BundlrTx;

#[cfg(feature = "ethereum")]
pub use signers::ethereum::EthereumSigner;

#[cfg(feature = "erc20")]
pub use signers::erc20::ERC20Signer;

pub use bundlr::Bundlr;
pub use index::JWK;
pub use signers::signer::{Signer, Verifier};
