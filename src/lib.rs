
extern crate derive_builder;
#[cfg(feature = "solana")]
extern crate ed25519_dalek;

mod signers;
pub mod error;
mod index;
mod transaction;
pub mod tags;
pub mod deep_hash;
pub mod deep_hash_sync;
// pub mod stream;
pub mod verify;
mod bundlr;

pub use transaction::BundlrTx;
#[cfg(feature = "solana")]
pub use signers::solana::SolanaSigner;

#[cfg(feature = "ethereum")]
pub use signers::ethereum::EthereumSigner;

pub use signers::signer::{Signer, Verifier};
pub use index::JWK;
pub use bundlr::Bundlr;