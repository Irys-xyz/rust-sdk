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
        .map(|balance| format!("{byte_amount} bytes in {token} is {balance} base units"))
}
