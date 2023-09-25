use std::{path::PathBuf, str::FromStr};

use crate::{
    bundlr::BundlrBuilder,
    consts::USE_JS_SDK,
    currency::{arweave::ArweaveBuilder, CurrencyType},
    error::BundlrError,
};
use num_traits::Zero;
use reqwest::Url;

pub async fn run_fund(
    amount: u64,
    url: Url,
    wallet: &str,
    currency: CurrencyType,
) -> Result<String, BundlrError> {
    if amount.is_zero() {
        return Err(BundlrError::InvalidAmount);
    }

    let wallet = PathBuf::from_str(wallet).expect("Invalid wallet path");
    match currency {
        CurrencyType::Arweave => {
            let currency = ArweaveBuilder::new().keypair_path(wallet).build()?;
            let bundlr = BundlrBuilder::new()
                .url(url)
                .currency(currency)
                .fetch_pub_info()
                .await?
                .build()?;
            bundlr.fund(amount, None).await.map(|res| res.to_string())
        }
        CurrencyType::Solana => todo!("{}", USE_JS_SDK),
        CurrencyType::Ethereum => todo!("{}", USE_JS_SDK),
        CurrencyType::Erc20 => todo!("{}", USE_JS_SDK),
        CurrencyType::Cosmos => todo!("{}", USE_JS_SDK),
    }
}
