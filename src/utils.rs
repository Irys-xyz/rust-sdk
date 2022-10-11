use reqwest::{Response, Url};
use serde::Deserialize;

use crate::error::BundlrError;

pub async fn check_and_return<T: for<'de> Deserialize<'de>>(
    res: Result<Response, reqwest::Error>,
) -> Result<T, BundlrError> {
    match res {
        Ok(r) => {
            if !r.status().is_success() {
                let msg = format!("Status: {}", r.status());
                return Err(BundlrError::ResponseError(msg));
            };
            Ok(r.json::<T>().await.expect("Could not convert to type"))
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
            .expect("Could not join url with /account/withdrawals/{}?address={}"),
        )
        .send()
        .await;
    check_and_return::<u64>(res).await
}
