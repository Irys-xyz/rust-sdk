use reqwest::Response;
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
