use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
    str::FromStr,
};

use crate::{
    bundlr::BundlrBuilder,
    consts::VERSION,
    currency::{
        arweave::ArweaveBuilder, ethereum::EthereumBuilder, solana::SolanaBuilder, CurrencyType,
    },
    error::BundlrError,
    tags::Tag,
};
use reqwest::Url;

pub async fn run_upload(
    file_path: String,
    url: Url,
    wallet: &str,
    currency: CurrencyType,
) -> Result<String, BundlrError> {
    let f = File::open(file_path.clone()).expect("Invalid file path");
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();

    // Read file into vector.
    reader.read_to_end(&mut buffer)?;

    let base_tag = Tag::new("User-Agent", &format!("bundlr-sdk-rs/{}", VERSION));

    match currency {
        CurrencyType::Arweave => {
            let wallet = PathBuf::from_str(wallet)
                .map_err(|err| BundlrError::ParseError(err.to_string()))?;
            let currency = ArweaveBuilder::new().keypair_path(wallet).build()?;
            let bundlr = BundlrBuilder::new()
                .url(url)
                .currency(currency)
                .fetch_pub_info()
                .await?
                .build()?;
            let mut tx = bundlr.create_transaction(buffer, vec![base_tag])?;
            let sig = bundlr.sign_transaction(&mut tx).await;
            assert!(sig.is_ok());
            match bundlr.send_transaction(tx).await {
                Ok(res) => Ok(format!("File {} uploaded: {:?}", file_path, res)),
                Err(err) => Err(BundlrError::UploadError(err.to_string())),
            }
        }
        CurrencyType::Solana => {
            let currency = SolanaBuilder::new().wallet(wallet).build()?;
            let bundlr = BundlrBuilder::new()
                .url(url)
                .currency(currency)
                .fetch_pub_info()
                .await?
                .build()?;
            let mut tx = bundlr.create_transaction(buffer, vec![base_tag])?;
            let sig = bundlr.sign_transaction(&mut tx).await;
            assert!(sig.is_ok());
            match bundlr.send_transaction(tx).await {
                Ok(res) => Ok(format!("File {} uploaded: {:?}", file_path, res)),
                Err(err) => Err(BundlrError::UploadError(err.to_string())),
            }
        }
        CurrencyType::Ethereum => {
            let currency = EthereumBuilder::new().wallet(wallet).build()?;
            let bundlr = BundlrBuilder::new()
                .url(url)
                .currency(currency)
                .fetch_pub_info()
                .await?
                .build()?;
            let mut tx = bundlr.create_transaction(buffer, vec![base_tag])?;
            let sig = bundlr.sign_transaction(&mut tx).await;
            assert!(sig.is_ok());
            match bundlr.send_transaction(tx).await {
                Ok(res) => Ok(format!("File {} uploaded: {:?}", file_path, res)),
                Err(err) => Err(BundlrError::UploadError(err.to_string())),
            }
        }
        CurrencyType::Erc20 => todo!(),
        CurrencyType::Cosmos => todo!(),
    }
}
