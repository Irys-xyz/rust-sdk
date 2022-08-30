use crate::{
    currency::{arweave::Arweave, CurrencyType},
    error::BundlrError,
    Bundlr,
};

pub async fn run_balance(
    url: &str,
    address: &str,
    currency: &CurrencyType,
) -> Result<String, BundlrError> {
    let currency = Arweave::new(None);
    let client = reqwest::Client::new();
    Bundlr::get_balance_public(&url, &currency, &address, &client)
        .await
        .map(|balance| balance.to_string())
}
