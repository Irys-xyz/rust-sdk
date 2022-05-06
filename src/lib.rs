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
extern crate ed25519_dalek;
#[cfg(feature = "solana")]
pub use signers::solana::SolanaSigner;

#[cfg(feature = "ethereum")]
pub use signers::ethereum::EthereumSigner;

#[cfg(feature = "erc20")]
pub use signers::erc20::ERC20Signer;

pub use bundlr::Bundlr;
pub use signers::signer::{Signer, Verifier};
