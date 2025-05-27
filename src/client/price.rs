use reqwest::Url;

use crate::{bundler::get_price, error::BundlerError, token::TokenType};

pub async fn run_price(
    url: Url,
    token: TokenType,
    byte_amount: u64,
) -> Result<String, BundlerError> {
    let client = reqwest::Client::new();
    get_price(&url, token, &client, byte_amount)
        .await
        .map(|balance| {
            format!(
                "{} bytes in {} is {} base units", //TODO: refactor this to show base unit name
                byte_amount, token, balance,
            )
        })
}
