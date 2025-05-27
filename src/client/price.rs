use reqwest::Url;

use crate::{bundler::get_price, currency::TokenType, error::BundlerError};

pub async fn run_price(
    url: Url,
    currency: TokenType,
    byte_amount: u64,
) -> Result<String, BundlerError> {
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
