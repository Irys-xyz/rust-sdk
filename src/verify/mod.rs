use bytes::Bytes;

use crate::error::BundlrError;

pub mod file;
pub mod types;

pub trait Verifier
where
    Self: Sized,
{
    fn verify(pk: Bytes, message: Bytes, signature: Bytes) -> Result<(), BundlrError>;
}
