use reqwest::Url;

use crate::{bundler::get_balance, error::BundlerError, token::TokenType};

pub async fn run_balance(
    url: Url,
    address: &str,
    token: TokenType,
) -> Result<String, BundlerError> {
    let client = reqwest::Client::new();
    get_balance(&url, token, address, &client)
        .await
        .map(|balance| balance.to_string())
}
