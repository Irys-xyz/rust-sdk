extern crate derive_builder;

mod bundlr;
mod signers;
mod transaction;

#[cfg(feature = "build-binary")]
pub mod client;

pub mod currency;
pub mod deep_hash;
pub mod deep_hash_sync;
pub mod error;
pub mod index;
pub mod tags;
pub mod verify;
pub mod wallet;

pub use bundlr::Bundlr;
pub use signers::Signer;
pub use verify::Verifier;
pub use transaction::BundlrTx;

#[cfg(feature = "arweave")]
pub use signers::arweave::ArweaveSigner;

#[cfg(any(feature = "solana", feature = "algorand"))]
pub use signers::ed25519::Ed25519Signer;

#[cfg(any(feature = "ethereum", feature = "erc20"))]
pub use signers::secp256k1::Secp256k1Signer;

#[cfg(feature = "cosmos")]
pub use signers::cosmos::CosmosSigner;
