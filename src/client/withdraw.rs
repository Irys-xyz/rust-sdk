use std::{path::PathBuf, str::FromStr};

use crate::{
    bundler::BundlerClientBuilder,
    consts::USE_JS_SDK,
    token::{arweave::ArweaveBuilder, TokenType},
    error::BundlerError,
};
use num_traits::Zero;
use reqwest::Url;

pub async fn run_withdraw(
    amount: u64,
    url: Url,
    wallet: &str,
    token: TokenType,
) -> Result<String, BundlerError> {
    if amount.is_zero() {
        return Err(BundlerError::InvalidAmount);
    }

    match token {
        TokenType::Arweave => {
            let wallet = PathBuf::from_str(wallet).expect("Invalid wallet path");
            let token = ArweaveBuilder::new().keypair_path(wallet).build()?;
            let bundler_client = BundlerClientBuilder::new()
                .url(url)
                .token(token)
                .fetch_pub_info()
                .await?
                .build()?;
            bundler_client
                .withdraw(amount)
                .await
                .map(|res| res.to_string())
        }
        TokenType::Solana => todo!("{}", USE_JS_SDK),
        TokenType::Ethereum => todo!("{}", USE_JS_SDK),
        TokenType::Erc20 => todo!("{}", USE_JS_SDK),
        TokenType::Cosmos => todo!("{}", USE_JS_SDK),
    }
}
