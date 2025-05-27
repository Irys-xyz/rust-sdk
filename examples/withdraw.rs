use std::{path::PathBuf, str::FromStr};

use irys_sdk::{bundler::ClientBuilder, currency::arweave::ArweaveBuilder, error::BundlerError};
use reqwest::Url;

#[tokio::main]
async fn main() -> Result<(), BundlerError> {
    let url = Url::parse("https://uploader.irys.xyz").unwrap();
    let wallet = PathBuf::from_str("res/test_wallet.json").unwrap();
    let currency = ArweaveBuilder::new()
        .keypair_path(wallet)
        .build()
        .expect("Could not create currency instance");
    let bundler_client = ClientBuilder::new()
        .url(url)
        .currency(currency)
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
