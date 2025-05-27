use std::{path::PathBuf, str::FromStr};

use irys_sdk::{
    bundler::BundlerClientBuilder, error::BundlerError, token::arweave::ArweaveBuilder,
};
use reqwest::Url;

#[tokio::main]
async fn main() -> Result<(), BundlerError> {
    let url = Url::parse("https://uploader.irys.xyz").unwrap();
    let wallet = PathBuf::from_str("res/test_wallet.json").unwrap();
    let token = ArweaveBuilder::new()
        .keypair_path(wallet)
        .build()
        .expect("Could not create token instance");
    let bundler_client = BundlerClientBuilder::new()
        .url(url)
        .token(token)
        .fetch_pub_info()
        .await?
        .build()?;

    let res = bundler_client
        .withdraw(10000)
        .await
        .map(|res| res.to_string());
    match res {
        Ok(ok) => println!("[ok] {}", ok),
        Err(err) => println!("[err] {}", err),
    }
    Ok(())
}
