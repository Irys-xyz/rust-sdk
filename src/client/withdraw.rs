use std::{path::PathBuf, str::FromStr};

use crate::{
    consts::USE_JS_SDK,
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

    let currency: Box<dyn Currency> = match currency {
        CurrencyType::Arweave => {
            let wallet = PathBuf::from_str(wallet).expect("Invalid wallet path");
            Box::new(Arweave::new(wallet, None)?)
        }
        CurrencyType::Solana => todo!("{}", USE_JS_SDK),
        CurrencyType::Ethereum => todo!("{}", USE_JS_SDK),
        CurrencyType::Erc20 => todo!("{}", USE_JS_SDK),
        CurrencyType::Cosmos => todo!("{}", USE_JS_SDK),
    };
    let bundlr = Bundlr::new(url, currency.as_ref()).await?;
    bundlr.withdraw(amount).await.map(|res| res.to_string())
}
