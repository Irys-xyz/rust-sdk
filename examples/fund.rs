use std::{path::PathBuf, str::FromStr};

use bundlr_sdk::{currency::arweave::ArweaveBuilder, Bundlr};
use reqwest::Url;

#[tokio::main]
async fn main() {
    let url = Url::parse("https://node1.bundlr.network").unwrap();
    let wallet = PathBuf::from_str("res/test_wallet.json").unwrap();
    let currency = ArweaveBuilder::new()
        .keypair_path(wallet)
        .build()
        .expect("Could not create currency instance");
    let bundlr = Bundlr::new(url, &currency)
        .await
        .expect("Could not create Bundlr");

    let res = bundlr.fund(10000, None).await.map(|res| res.to_string());
    match res {
        Ok(ok) => println!("[ok] {}", ok),
        Err(err) => println!("[err] {}", err),
    }
}
