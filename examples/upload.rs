use std::{path::PathBuf, str::FromStr};

use bundlr_sdk::{currency::solana::Solana, Bundlr};
use reqwest::Url;

#[tokio::main]
async fn main() {
    let url = Url::parse("https://node1.bundlr.network").unwrap();
    let currency = Solana::new(
        "kNykCXNxgePDjFbDWjPNvXQRa8U12Ywc19dFVaQ7tebUj3m7H4sF4KKdJwM7yxxb3rqxchdjezX9Szh8bLcQAjb",
        None,
    );
    let mut bundlr = Bundlr::new(url, &currency).await;

    let file = PathBuf::from_str("res/test_image.jpg").unwrap();
    let res = bundlr.upload_file(file).await;
    match res {
        Ok(()) => println!("[ok]"),
        Err(err) => println!("[err] {}", err),
    }
}
