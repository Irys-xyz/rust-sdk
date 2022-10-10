use std::{path::PathBuf, str::FromStr};

use crate::{
    currency::{arweave::Arweave, Currency, CurrencyType},
    error::BundlrError,
    Bundlr,
};
use num_traits::Zero;
use reqwest::Url;

pub async fn run_withdraw(
    amount: u64,
    url: Url,
    wallet: &str,
    currency: CurrencyType,
) -> Result<String, BundlrError> {
    if amount.is_zero() {
        return Err(BundlrError::InvalidAmount);
    }

    let wallet = PathBuf::from_str(wallet).expect("Invalid wallet path");
    let currency: Box<dyn Currency> = match currency {
        CurrencyType::Arweave => Box::new(Arweave::new(wallet, None)),
        CurrencyType::Solana => todo!(),
        CurrencyType::Ethereum => todo!(),
        CurrencyType::Erc20 => todo!(),
        CurrencyType::Cosmos => todo!(),
    };
    let bundlr = Bundlr::new(url, currency.as_ref()).await;
    bundlr.withdraw(amount).await.map(|res| res.to_string())
}
