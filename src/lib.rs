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

#[cfg(any(feature = "solana", feature = "algorand"))]
pub use signers::ed25519::Ed25519Signer;

#[cfg(any(feature = "ethereum", feature = "erc20"))]
pub use signers::secp256k1::Secp256k1Signer;

#[cfg(feature = "cosmos")]
pub use signers::cosmos::CosmosSigner;

pub use bundlr::Bundlr;
pub use signers::signer::{Signer, Verifier};
