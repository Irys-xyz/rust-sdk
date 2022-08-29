use clap::ArgEnum;

use crate::{currency::Currency, error::BundlrError, Bundlr};

pub async fn run_balance(url: &str, address: &str, currency: &str) -> Result<String, BundlrError> {
    let currency = Currency::from_str(currency, false).unwrap();
    let client = reqwest::Client::new();
    Bundlr::get_balance_public(&url, &currency, &address, &client)
        .await
        .map(|balance| balance.to_string())
}
