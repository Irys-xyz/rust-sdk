use reqwest::Url;

use crate::{bundlr::get_price, currency::CurrencyType, error::BundlrError};

pub async fn run_price(
    url: Url,
    currency: CurrencyType,
    byte_amount: u64,
) -> Result<String, BundlrError> {
    let client = reqwest::Client::new();
    get_price(&url, currency, &client, byte_amount)
        .await
        .map(|balance| {
            format!(
                "{} bytes in {} is {} base units", //TODO: refactor this to show base unit name
                byte_amount, currency, balance,
            )
        })
}
