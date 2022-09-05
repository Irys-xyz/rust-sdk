use crate::{error::BundlrError, index::SignerMap};
use bytes::Bytes;

pub trait ToPem {}

pub trait Signer {
    fn sign(&self, message: Bytes) -> Result<Bytes, BundlrError>;
    fn sig_type(&self) -> SignerMap;
    fn get_sig_length(&self) -> u16;
    fn get_pub_length(&self) -> u16;
    fn pub_key(&self) -> Bytes;
}

pub trait Verifier
where
    Self: Sized,
{
    fn verify(pk: Bytes, message: Bytes, signature: Bytes) -> Result<bool, BundlrError>;
}
