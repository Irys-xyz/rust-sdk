use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum BundlrError {
    InvalidHeaders,
    InvalidSignerType,
    InvalidPresenceByte,
    NoBytesLeft,
    InvalidTagEncoding,
    FsError,
    InvalidSignature,
    ResponseError
}