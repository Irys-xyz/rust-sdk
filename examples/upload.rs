use bundlr_sdk::{bundlr::BundlrBuilder, currency::solana::SolanaBuilder, error::BundlrError};
use reqwest::Url;
use std::{path::PathBuf, str::FromStr};

#[tokio::main]
async fn main() -> Result<(), BundlrError> {
    let url = Url::parse("https://uploader.irys.xyz").unwrap();
    let currency = SolanaBuilder::new().wallet(
        "kNykCXNxgePDjFbDWjPNvXQRa8U12Ywc19dFVaQ7tebUj3m7H4sF4KKdJwM7yxxb3rqxchdjezX9Szh8bLcQAjb")
        .build()
        .expect("Could not create Solana instance");
    let mut bundlr = BundlrBuilder::new()
        .url(url)
        .currency(currency)
        .fetch_pub_info()
        .await?
        .build()?;

    let file = PathBuf::from_str("res/test_image.jpg").unwrap();
    let res = bundlr.upload_file(file).await;
    match res {
        Ok(res) => println!("Uploaded to  https://uploader.irys.xyz/tx/{}", &res.id),
        Err(err) => println!("[err] {}", err),
    }
    Ok(())
}
