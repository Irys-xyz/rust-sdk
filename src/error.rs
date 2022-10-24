use thiserror::Error;

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

    #[error("Upload error: {0}")]
    UploadError(String),
}
