use crate::{
    currency::{arweave::Arweave, Currency, CurrencyType},
    error::BundlrError,
    Bundlr,
};

pub async fn run_balance(
    url: &str,
    address: &str,
    currency: &CurrencyType,
) -> Result<String, BundlrError> {
    let currency: Box<dyn Currency> = match currency {
        CurrencyType::Arweave => Box::new(Arweave::new(None)),
        CurrencyType::Solana => todo!(),
        CurrencyType::Ethereum => todo!(),
        CurrencyType::Erc20 => todo!(),
        CurrencyType::Cosmos => todo!(),
    };
    let client = reqwest::Client::new();
    Bundlr::get_balance_public(&url, currency.as_ref(), &address, &client)
        .await
        .map(|balance| balance.to_string())
}
