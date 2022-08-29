use thiserror::Error;

#[derive(Debug, Error)]
pub enum BundlrError {
    #[error("Invalid headers provided.")]
    InvalidHeaders,

    #[error("Invalid signer type used.")]
    InvalidSignerType,

    #[error("Invalid presence byte.")]
    InvalidPresenceByte,

    #[error("No bytes left.")]
    NoBytesLeft,

    #[error("Invalid tag encoding.")]
    InvalidTagEncoding,

    #[error("File system error: {0}")]
    FsError(String),

    #[error("Invalid signature.")]
    InvalidSignature,

    #[error("Invalid value for funding.")]
    InvalidFundingValue,

    #[error("Invalid amount, must be a integer bigger than zero")]
    InvalidAmount,

    #[error("Invalid wallet")]
    InvalidWallet,

    #[error("Invalid currency: {0}")]
    InvalidCurrency(String),

    #[error("Response failed with the following error: {0}")]
    ResponseError(String),

    #[error("Failed to sign message: {0}")]
    SigningError(String),

    #[error("Request error: {0}.")]
    RequestError(String),
}
