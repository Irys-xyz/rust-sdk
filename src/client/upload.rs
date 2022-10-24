use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::PathBuf,
    str::FromStr,
};

use crate::{
    consts::VERSION,
    currency::{arweave::Arweave, Currency, CurrencyType},
    error::BundlrError,
    tags::Tag,
    Bundlr,
};
use reqwest::Url;

pub async fn run_upload(
    file_path: String,
    url: Url,
    wallet: &str,
    currency: CurrencyType,
) -> Result<String, BundlrError> {
    let wallet = PathBuf::from_str(&wallet).expect("Invalid wallet path");
    let currency: Box<dyn Currency> = match currency {
        CurrencyType::Arweave => Box::new(Arweave::new(wallet, None)),
        CurrencyType::Solana => todo!(),
        CurrencyType::Ethereum => todo!(),
        CurrencyType::Erc20 => todo!(),
        CurrencyType::Cosmos => todo!(),
    };
    let f = File::open(file_path.clone()).expect("Invalid file path");
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();

    // Read file into vector.
    reader.read_to_end(&mut buffer)?;

    let base_tag = Tag::new(
        "User-Agent".to_string(),
        format!("bundlr-sdk-rs/{}", VERSION),
    );

    // Read.
    let bundlr = Bundlr::new(url, currency.as_ref()).await;
    let mut tx = bundlr.create_transaction(buffer, vec![base_tag]);
    let sig = bundlr.sign_transaction(&mut tx).await;
    assert!(sig.is_ok());
    match bundlr.send_transaction(tx).await {
        Ok(res) => Ok(format!("File {} uploaded: {}", file_path, res.to_string())),
        Err(err) => Err(BundlrError::UploadError(err.to_string())),
    }
}
