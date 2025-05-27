use std::{path::PathBuf, str::FromStr};

use crate::{
    bundler::ClientBuilder,
    consts::USE_JS_SDK,
    currency::{arweave::ArweaveBuilder, TokenType},
    error::BundlerError,
};
use num_traits::Zero;
use reqwest::Url;

pub async fn run_fund(
    amount: u64,
    url: Url,
    wallet: &str,
    currency: TokenType,
) -> Result<String, BundlerError> {
    if amount.is_zero() {
        return Err(BundlerError::InvalidAmount);
    }

    let wallet = PathBuf::from_str(wallet).expect("Invalid wallet path");
    match currency {
        TokenType::Arweave => {
            let currency = ArweaveBuilder::new().keypair_path(wallet).build()?;
            let bundler_client = ClientBuilder::new()
                .url(url)
                .currency(currency)
                .fetch_pub_info()
                .await?
                .build()?;
            bundler_client
                .fund(amount, None)
                .await
                .map(|res| res.to_string())
        }
        TokenType::Solana => todo!("{}", USE_JS_SDK),
        TokenType::Ethereum => todo!("{}", USE_JS_SDK),
        TokenType::Erc20 => todo!("{}", USE_JS_SDK),
        TokenType::Cosmos => todo!("{}", USE_JS_SDK),
    }
}
