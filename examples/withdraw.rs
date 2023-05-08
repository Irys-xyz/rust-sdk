use std::{path::PathBuf, str::FromStr};

use bundlr_sdk::{
    bundlr::BundlrBuilder,
    currency::arweave::{Arweave, ArweaveBuilder},
    error::BundlrError,
};
use reqwest::Url;

#[tokio::main]
async fn main() -> Result<(), BundlrError> {
    let url = Url::parse("https://node1.bundlr.network").unwrap();
    let wallet = PathBuf::from_str("res/test_wallet.json").unwrap();
    let currency = ArweaveBuilder::new()
        .keypair_path(wallet)
        .build()
        .expect("Could not create currency instance");
    let bundlr = BundlrBuilder::<Arweave>::new()
        .url(url)
        .currency(currency)
        .fetch_pub_info()
        .await?
        .build()?;

    let res = bundlr.withdraw(10000).await.map(|res| res.to_string());
    match res {
        Ok(ok) => println!("[ok] {}", ok),
        Err(err) => println!("[err] {}", err),
    }
    Ok(())
}
