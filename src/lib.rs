extern crate derive_builder;

mod signers;
mod transaction;

#[cfg(feature = "build-binary")]
pub mod client;

pub mod bundler;
pub mod consts;
pub mod deep_hash;
pub mod deep_hash_sync;
pub mod error;
pub mod index;
pub mod tags;
pub mod token;
pub mod upload;
pub mod utils;
pub mod verify;

pub use bundler::{BundlerClient, BundlerClientBuilder};
pub use signers::Signer;
pub use transaction::irys::BundlerTx;
pub use verify::Verifier;

#[cfg(feature = "arweave")]
pub use signers::arweave::ArweaveSigner;

#[cfg(any(feature = "solana", feature = "algorand"))]
pub use signers::ed25519::Ed25519Signer;

#[cfg(any(feature = "ethereum", feature = "erc20"))]
pub use signers::secp256k1::Secp256k1Signer;

#[cfg(feature = "cosmos")]
pub use signers::cosmos::CosmosSigner;

#[cfg(feature = "aptos")]
pub use signers::aptos::AptosSigner;

#[cfg(feature = "aptos")]
pub use signers::aptos::MultiAptosSigner;
