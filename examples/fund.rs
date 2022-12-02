use std::{path::PathBuf, str::FromStr};

use bundlr_sdk::{currency::arweave::Arweave, Bundlr};
use reqwest::Url;

#[tokio::main]
async fn main() {
    let url = Url::parse("https://node1.bundlr.network").unwrap();
    let wallet = PathBuf::from_str("res/test_wallet.json").unwrap();
    let currency = Arweave::new(wallet, None);
    let bundlr = Bundlr::new(url, &currency).await;

    let res = bundlr.fund(10000, None).await.map(|res| res.to_string());
    match res {
        Ok(ok) => println!("[ok] {}", ok),
        Err(err) => println!("[err] {}", err),
    }
}
