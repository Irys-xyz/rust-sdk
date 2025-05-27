use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
    str::FromStr,
};

use crate::{
    bundler::BundlerClientBuilder,
    consts::VERSION,
    token::{
        arweave::ArweaveBuilder, ethereum::EthereumBuilder, solana::SolanaBuilder, TokenType,
    },
    error::BundlerError,
    tags::Tag,
};
use reqwest::Url;

pub async fn run_upload(
    file_path: String,
    url: Url,
    wallet: &str,
    token: TokenType,
) -> Result<String, BundlerError> {
    let f = File::open(file_path.clone()).expect("Invalid file path");
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();

    // Read file into vector.
    reader.read_to_end(&mut buffer)?;

    let base_tag = Tag::new("User-Agent", &format!("irys-bundler-sdk-rs/{}", VERSION));

    match token {
        TokenType::Arweave => {
            let wallet = PathBuf::from_str(wallet)
                .map_err(|err| BundlerError::ParseError(err.to_string()))?;
            let token = ArweaveBuilder::new().keypair_path(wallet).build()?;
            let bundler_client = BundlerClientBuilder::new()
                .url(url)
                .token(token)
                .fetch_pub_info()
                .await?
                .build()?;
            let mut tx = bundler_client.create_transaction(buffer, vec![base_tag])?;
            let sig = bundler_client.sign_transaction(&mut tx).await;
            assert!(sig.is_ok());
            match bundler_client.send_transaction(tx).await {
                Ok(res) => Ok(format!("File {} uploaded: {:?}", file_path, res)),
                Err(err) => Err(BundlerError::UploadError(err.to_string())),
            }
        }
        TokenType::Solana => {
            let token = SolanaBuilder::new().wallet(wallet).build()?;
            let bundler_client = BundlerClientBuilder::new()
                .url(url)
                .token(token)
                .fetch_pub_info()
                .await?
                .build()?;
            let mut tx = bundler_client.create_transaction(buffer, vec![base_tag])?;
            let sig = bundler_client.sign_transaction(&mut tx).await;
            assert!(sig.is_ok());
            match bundler_client.send_transaction(tx).await {
                Ok(res) => Ok(format!("File {} uploaded: {:?}", file_path, res)),
                Err(err) => Err(BundlerError::UploadError(err.to_string())),
            }
        }
        TokenType::Ethereum => {
            let token = EthereumBuilder::new().wallet(wallet).build()?;
            let bundler_client = BundlerClientBuilder::new()
                .url(url)
                .token(token)
                .fetch_pub_info()
                .await?
                .build()?;
            let mut tx = bundler_client.create_transaction(buffer, vec![base_tag])?;
            let sig = bundler_client.sign_transaction(&mut tx).await;
            assert!(sig.is_ok());
            match bundler_client.send_transaction(tx).await {
                Ok(res) => Ok(format!("File {} uploaded: {:?}", file_path, res)),
                Err(err) => Err(BundlerError::UploadError(err.to_string())),
            }
        }
        TokenType::Erc20 => todo!(),
        TokenType::Cosmos => todo!(),
    }
}
