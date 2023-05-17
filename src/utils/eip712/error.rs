use thiserror::Error;
use validator::{ValidationError, ValidationErrors};

/// Possible errors encountered while hashing/encoding an EIP-712 compliant data structure
#[derive(Clone, Debug, PartialEq, Error)]
pub enum Eip712Error {
    /// if we fail to deserialize from a serde::Value as a type specified in message types
    /// fail with this error.
    #[error("Expected type '{0}' for field '{1}'")]
    UnexpectedType(String, String),
    /// the primary type supplied doesn't exist in the MessageTypes
    #[error("The given primaryType wasn't found in the types field")]
    NonExistentType,
    /// an invalid address was encountered during encoding
    #[error("Address string should be a 0x-prefixed 40 character string, got '{0}'")]
    InvalidAddressLength(usize),
    /// a hex parse error occured
    #[error("Failed to parse hex '{0}'")]
    HexParseError(String),
    /// the field was declared with a unknown type
    #[error("The field '{0}' has an unknown type '{1}'")]
    UnknownType(String, String),
    /// Unexpected token
    #[error("Unexpected token '{0}' while parsing typename '{1}'")]
    UnexpectedToken(String, String),
    /// the user has attempted to define a typed array with a depth > 10
    #[error("Maximum depth for nested arrays is 10")]
    UnsupportedArrayDepth,
    /// FieldType validation error
    #[error("{0}")]
    ValidationError(ValidationError),
    #[error("{0}")]
    ValidationErrors(ValidationErrors),
    /// the typed array defined in message types was declared with a fixed length
    /// that is of unequal length with the items to be encoded
    #[error("Expected {0} items for array type {1}, got {2} items")]
    UnequalArrayItems(u64, String, u64),
    /// Typed array length doesn't fit into a u64
    #[error("Attempted to declare fixed size with length {0}")]
    InvalidArraySize(String),
}

pub(crate) fn serde_error(expected: &str, field: Option<&str>) -> Eip712Error {
    Eip712Error::UnexpectedType(expected.to_owned(), field.unwrap_or("").to_owned())
}
