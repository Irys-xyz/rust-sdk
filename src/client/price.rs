use reqwest::Url;

use crate::{
    currency::{arweave::Arweave, solana::Solana, Currency, CurrencyType},
    error::BundlrError,
    Bundlr,
};

pub async fn run_price(
    url: Url,
    currency: CurrencyType,
    byte_amount: u64,
) -> Result<String, BundlrError> {
    let currency: Box<dyn Currency> = match currency {
        CurrencyType::Arweave => Box::new(Arweave::default()),
        CurrencyType::Solana => Box::new(Solana::default()),
        CurrencyType::Ethereum => todo!(),
        CurrencyType::Erc20 => todo!(),
        CurrencyType::Cosmos => todo!(),
    };
    let client = reqwest::Client::new();
    Bundlr::get_price_public(&url, currency.as_ref(), &client, byte_amount)
        .await
        .map(|balance| {
            format!(
                "{} bytes in {} is {} {}",
                byte_amount,
                currency.get_type(),
                balance.to_string(),
                currency.get_min_unit_name()
            )
        })
}
