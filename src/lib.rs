extern crate derive_builder;

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

pub use signers::arweave::ArweaveSigner;
pub use transaction::BundlrTx;

#[cfg(feature = "solana")]
pub use signers::solana::SolanaSigner;

#[cfg(feature = "ethereum")]
pub use signers::ethereum::EthereumSigner;

#[cfg(feature = "cosmos")]
pub use signers::cosmos::CosmosSigner;

pub use bundlr::Bundlr;
pub use signers::signer::{Signer, Verifier};
