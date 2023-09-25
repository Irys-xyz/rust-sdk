use reqwest::Url;

use crate::{bundlr::get_balance, currency::CurrencyType, error::BundlrError};

pub async fn run_balance(
    url: Url,
    address: &str,
    currency: CurrencyType,
) -> Result<String, BundlrError> {
    let client = reqwest::Client::new();
    get_balance(&url, currency, address, &client)
        .await
        .map(|balance| balance.to_string())
}
