use crate::error::BundlrError;
use bytes::Bytes;

pub trait ToPem {}

pub trait Signer
where
    Self: Sized,
{
    const SIG_TYPE: u16;
    const SIG_LENGTH: u16;
    const PUB_LENGTH: u16;
    fn sign(&self, message: Bytes) -> Result<Bytes, BundlrError>;
    fn sig_type(&self) -> u16 {
        Self::SIG_TYPE
    }
    fn get_sig_length(&self) -> u16 {
        Self::SIG_LENGTH
    }
    fn get_pub_length(&self) -> u16 {
        Self::PUB_LENGTH
    }
    fn pub_key(&self) -> Bytes;
}

pub trait Verifier
where
    Self: Sized,
{
    fn verify(pk: Bytes, message: Bytes, signature: Bytes) -> Result<bool, BundlrError>;
}
