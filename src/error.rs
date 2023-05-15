use thiserror::Error;
use web3::signing::RecoveryError;

use crate::utils::Eip712Error;

#[derive(Debug, Error)]
pub enum BundlrError {
    #[error("Invalid headers provided.")]
    InvalidHeaders,

    #[error("Invalid signer type used.")]
    InvalidSignerType,

    #[error("Invalid presence byte {0}")]
    InvalidPresenceByte(String),

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

    #[error("Invalid wallet {0}")]
    InvalidKey(String),

    #[error("Invalid currency: {0}")]
    InvalidCurrency(String),

    #[error("Response failed with the following error: {0}")]
    ResponseError(String),

    #[error("Failed to sign message: {0}")]
    SigningError(String),

    #[error("Request error: {0}.")]
    RequestError(String),

    #[error("Tx not found")]
    TxNotFound,

    #[error("Tx status not confirmed")]
    TxStatusNotConfirmed,

    #[error("Chunk size out of allowed range: {0} - {0}")]
    ChunkSizeOutOfRange(u64, u64),

    #[error("Error posting chunk: {0}")]
    PostChunkError(String),

    #[error("No signature present")]
    NoSignature,

    #[error("Cannot convert file stream to known bytes. Try using another method")]
    InvalidDataType,

    #[error("Arweave Sdk error: {0}")]
    ArweaveSdkError(arweave_rs::error::Error),

    #[error("Currency error: {0}")]
    CurrencyError(String),

    #[error("Error reading/writting bytes: {0}")]
    BytesError(String),

    #[error("Error converting type: {0}")]
    TypeParseError(String),

    #[error("Parse error error: {0}")]
    ParseError(String),

    #[error("Upload error: {0}")]
    UploadError(String),

    #[error("Unknown: {0}")]
    Unknown(String),

    #[error("Unsupported: {0}")]
    Unsupported(String),

    #[error("ED25519 error: {0}")]
    ED25519Error(ed25519_dalek::ed25519::Error),

    #[error("Secp256k1 error: {0}")]
    Secp256k1Error(secp256k1::Error),

    #[error("Base64 error: {0}")]
    Base64Error(String),

    #[error("Io error: {0}")]
    IoError(std::io::Error),

    #[error("Builder error: {0}")]
    BuilderError(BuilderError),

    #[error("Eip712 error: {0}")]
    Eip712Error(Eip712Error),

    #[error("RecoveryError")]
    RecoveryError(RecoveryError),
}

impl From<BuilderError> for BundlrError {
    fn from(value: BuilderError) -> Self {
        Self::BuilderError(value)
    }
}

impl From<arweave_rs::error::Error> for BundlrError {
    fn from(value: arweave_rs::error::Error) -> Self {
        Self::ArweaveSdkError(value)
    }
}

#[derive(Debug, Error)]
pub enum BuilderError {
    #[error("Bundlr Error {0}")]
    BundlrError(String),

    #[error("Missing field {0}")]
    MissingField(String),

    #[error("Fetch pub info error: {0}")]
    FetchPubInfoError(String),

    #[error("Arweave Sdk error: {0}")]
    ArweaveSdkError(arweave_rs::error::Error),
}

impl From<arweave_rs::error::Error> for BuilderError {
    fn from(value: arweave_rs::error::Error) -> Self {
        Self::ArweaveSdkError(value)
    }
}

impl From<BundlrError> for BuilderError {
    fn from(value: BundlrError) -> Self {
        Self::BundlrError(value.to_string())
    }
}
