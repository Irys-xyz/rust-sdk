use reqwest::Url;

use crate::{bundler::get_balance, currency::TokenType, error::BundlerError};

pub async fn run_balance(
    url: Url,
    address: &str,
    currency: TokenType,
) -> Result<String, BundlerError> {
    let client = reqwest::Client::new();
    get_balance(&url, currency, address, &client)
        .await
        .map(|balance| balance.to_string())
}
