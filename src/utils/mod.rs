#[cfg(any(feature = "ethereum", feature = "erc20"))]
mod eip712;

pub(crate) use eip712::hash_structured_data;
pub(crate) use eip712::Eip712Error;
pub(crate) use eip712::EIP712;

use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use bytes::Bytes;
use reqwest::{Response, Url};
use serde::Deserialize;

use crate::error::BundlrError;

pub async fn check_and_return<T: for<'de> Deserialize<'de>>(
    res: Result<Response, reqwest::Error>,
) -> Result<T, BundlrError>
where
    T: Default,
{
    match res {
        Ok(r) => {
            if !r.status().is_success() {
                let status = r.status();
                let text = r
                    .text()
                    .await
                    .map_err(|err| BundlrError::ParseError(err.to_string()))?
                    .replace('\"', "");
                let msg = format!("Status: {}:{:?}", status, text);
                return Err(BundlrError::ResponseError(msg));
            };
            Ok(r.json::<T>().await.unwrap_or_default())
        }
        Err(err) => Err(BundlrError::ResponseError(err.to_string())),
    }
}

pub async fn get_nonce(
    client: &reqwest::Client,
    url: &Url,
    address: String,
    currency: String,
) -> Result<u64, BundlrError> {
    let res = client
        .get(
            url.join(&format!(
                "/account/withdrawals/{}?address={}",
                currency, address
            ))
            .map_err(|err| BundlrError::ParseError(err.to_string()))?,
        )
        .send()
        .await;
    check_and_return::<u64>(res).await
}

// Reads `length` bytes at `offset` within `file`
#[allow(clippy::uninit_vec)]
#[allow(clippy::unused_io_amount)]
pub fn read_offset(file: &mut File, offset: u64, length: usize) -> Result<Bytes, std::io::Error> {
    let mut b = Vec::with_capacity(length);
    unsafe { b.set_len(length) };
    file.seek(SeekFrom::Start(offset))?;

    b.fill(0);

    file.read(&mut b)?;
    Ok(b.into())
}
