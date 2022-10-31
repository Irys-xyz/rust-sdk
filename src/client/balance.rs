use reqwest::Url;

use crate::{
    currency::{arweave::Arweave, ethereum::Ethereum, solana::Solana, Currency, CurrencyType},
    error::BundlrError,
    Bundlr,
};

pub async fn run_balance(
    url: Url,
    address: &str,
    currency: CurrencyType,
) -> Result<String, BundlrError> {
    let client = reqwest::Client::new();
    Bundlr::get_balance_public(&url, currency, &address, &client)
        .await
        .map(|balance| balance.to_string())
}
