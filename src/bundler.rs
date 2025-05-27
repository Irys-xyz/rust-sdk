use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use crate::consts::DEFAULT_BUNDLER_URL;
use crate::currency;
use crate::currency::TokenType;
use crate::deep_hash::{deep_hash, DeepHashChunk};
use crate::error::{BuilderError, BundlerError};
use crate::tags::Tag;
use crate::upload::Uploader;
use crate::utils::{check_and_return, get_nonce};
use crate::BundlerTx;
use arweave_rs::crypto::base64::Base64;
use bytes::Bytes;
use num::BigUint;
use num::FromPrimitive;
use num_traits::Zero;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[allow(unused)]
pub struct IrysBundlerClient<Currency> {
    url: Url,
    currency: Currency,
    client: reqwest::Client,
    pub_info: PubInfo,
    uploader: Uploader,
}
#[allow(unused)]
#[derive(Deserialize, Default)]
pub struct PubInfo {
    version: String,
    gateway: String,
    addresses: HashMap<String, String>,
}
#[derive(Deserialize, Default)]
pub struct BalanceResData {
    balance: String,
}

#[derive(Serialize, Deserialize)]
pub struct FundBody {
    tx_id: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadReponse {
    pub block: u64,
    pub id: String,
    pub public: String,
    pub signature: String,
    pub timestamp: u64,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WithdrawBody {
    public_key: Base64,
    currency: String,
    amount: String,
    nonce: u64,
    signature: Base64,
    sig_type: u16,
}

#[derive(Default)]

pub struct ClientBuilder<Currency = ()> {
    url: Option<Url>,
    currency: Currency,
    client: Option<reqwest::Client>,
    pub_info: Option<PubInfo>,
}

impl ClientBuilder {
    pub fn new() -> ClientBuilder {
        Default::default()
    }
}

impl<Currency> ClientBuilder<Currency> {
    pub fn url(mut self, url: Url) -> ClientBuilder<Currency> {
        self.url = Some(url);
        self
    }

    pub fn client(mut self, client: reqwest::Client) -> ClientBuilder<Currency> {
        self.client = Some(client);
        self
    }

    pub async fn fetch_pub_info(mut self) -> Result<ClientBuilder<Currency>, BuilderError> {
        if let Some(url) = &self.url {
            let pub_info = match get_pub_info(url).await {
                Ok(info) => info,
                Err(err) => {
                    return Err(BuilderError::FetchPubInfoError(err.to_string()));
                }
            };
            self.pub_info = Some(pub_info);
            Ok(self)
        } else {
            Err(BuilderError::MissingField("url".to_owned()))
        }
    }

    pub fn pub_info(mut self, pub_info: PubInfo) -> ClientBuilder<Currency> {
        self.pub_info = Some(pub_info);
        self
    }
}

impl ClientBuilder<()> {
    pub fn currency<Currency>(self, currency: Currency) -> ClientBuilder<Currency>
    where
        Currency: currency::Currency,
    {
        ClientBuilder {
            currency,
            url: self.url,
            client: self.client,
            pub_info: self.pub_info,
        }
    }
}

impl<Currency> ClientBuilder<Currency>
where
    Currency: currency::Currency,
{
    pub fn build(self) -> Result<IrysBundlerClient<Currency>, BuilderError> {
        let url = self.url.unwrap_or(Url::parse(DEFAULT_BUNDLER_URL).unwrap());

        let client = self.client.unwrap_or_else(reqwest::Client::new);

        let pub_info = match self.pub_info {
            Some(p) => p,
            None => return Err(BuilderError::MissingField("currency".to_owned())),
        };

        let uploader = Uploader::new(url.clone(), client.clone(), self.currency.get_type());

        Ok(IrysBundlerClient {
            url,
            currency: self.currency,
            client,
            pub_info,
            uploader,
        })
    }
}

/// Gets the public info from a Irys bundler node.
///
/// # Examples
///
/// ```
/// # use irys_sdk::bundler::get_pub_info;
/// # use reqwest::Url;
/// # tokio_test::block_on(async {
/// let url = Url::parse("https://uploader.irys.xyz/").unwrap();
/// let res = get_pub_info(&url).await;
/// # });
/// ```
pub async fn get_pub_info(url: &Url) -> Result<PubInfo, BundlerError> {
    let client = reqwest::Client::new();
    let response = client
        .get(
            url.join("info")
                .map_err(|err| BundlerError::ParseError(err.to_string()))?,
        )
        .header("Content-Type", "application/json")
        .send()
        .await;

    check_and_return::<PubInfo>(response).await
}

/// Get balance from address in a Irys bundler node
pub async fn get_balance(
    url: &Url,
    currency: TokenType,
    address: &str,
    client: &reqwest::Client,
) -> Result<BigUint, BundlerError> {
    let response = client
        .get(
            url.join(&format!(
                "account/balance/{}",
                currency.to_string().to_lowercase()
            ))
            .map_err(|err| BundlerError::ParseError(err.to_string()))?,
        )
        .query(&[("address", address)])
        .header("Content-Type", "application/json")
        .send()
        .await;

    match check_and_return::<BalanceResData>(response).await {
        Ok(d) => match BigUint::from_str(&d.balance) {
            Ok(ok) => Ok(ok),
            Err(err) => Err(BundlerError::TypeParseError(err.to_string())),
        },
        Err(err) => Err(BundlerError::TypeParseError(err.to_string())),
    }
}

/// Get the cost for determined amount of bytes, measured in the currency's base units (i.e Winston for Arweave, or Lamport for Solana)
pub async fn get_price(
    url: &Url,
    currency: TokenType,
    client: &reqwest::Client,
    byte_amount: u64,
) -> Result<BigUint, BundlerError> {
    let response = client
        .get(
            url.join(&format!("/price/{}/{}", currency, byte_amount))
                .map_err(|err| BundlerError::ParseError(err.to_string()))?,
        )
        .header("Content-Type", "application/json")
        .send()
        .await;

    match check_and_return::<u64>(response).await {
        Ok(d) => match BigUint::from_u64(d) {
            Some(ok) => Ok(ok),
            None => Err(BundlerError::TypeParseError(
                "Could not parse u64 to BigUInt".to_owned(),
            )),
        },
        Err(err) => Err(err),
    }
}

impl<Currency> IrysBundlerClient<Currency>
where
    Currency: currency::Currency,
{
    /// Creates an unsigned transaction for posting.
    ///
    /// # Examples
    ///
    /// ```
    /// # use irys_sdk::{
    /// #   currency::TokenType,
    /// #   ClientBuilder,
    /// #   currency::arweave::ArweaveBuilder,
    /// #   tags::Tag,
    /// #   error::BuilderError
    /// # };
    /// # use std::{path::PathBuf, str::FromStr};
    /// # use reqwest::Url;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), BuilderError> {
    /// #   let url = Url::parse("https://uploader.irys.xyz").unwrap();
    /// #   let wallet = PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
    /// #   let currency = ArweaveBuilder::new()
    /// #       .keypair_path(wallet)
    /// #       .build()
    /// #       .expect("Could not create currency instance");
    /// #   let mut bundler_client = ClientBuilder::new()
    /// #       .url(url)
    /// #       .currency(currency)
    /// #       .fetch_pub_info()
    /// #       .await?
    /// #       .build()?;
    /// let data = b"Hello".to_vec();
    /// let tags = vec![Tag::new("name", "value")];
    /// let tx = bundler_client.create_transaction(data, tags).unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub fn create_transaction(
        &self,
        data: Vec<u8>,
        additional_tags: Vec<Tag>,
    ) -> Result<BundlerTx, BundlerError> {
        BundlerTx::new(vec![], data, additional_tags)
    }

    /// Signs a transaction
    ///
    /// # Examples
    ///
    /// ```
    /// # use irys_sdk::{
    /// #   currency::TokenType,
    /// #   ClientBuilder,
    /// #   currency::arweave::ArweaveBuilder,
    /// #   tags::Tag,
    /// #   error::BuilderError
    /// # };
    /// # use std::{path::PathBuf, str::FromStr};
    /// # use reqwest::Url;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), BuilderError> {
    /// #   let url = Url::parse("https://uploader.irys.xyz").unwrap();
    /// #   let wallet = PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
    /// #   let currency = ArweaveBuilder::new()
    /// #      .keypair_path(wallet)
    /// #      .build()
    /// #      .expect("Could not create currency instance");
    /// #   let mut bundler_client = ClientBuilder::new()
    /// #       .url(url)
    /// #       .currency(currency)
    /// #       .fetch_pub_info()
    /// #       .await?
    /// #       .build()?;
    /// let data = b"Hello".to_vec();
    /// # let data = b"Hello".to_vec();
    /// # let tags = vec![Tag::new("name", "value")];
    /// let mut tx = bundler_client.create_transaction(data, tags).unwrap();
    /// let sig = bundler_client.sign_transaction(&mut tx).await;
    /// # assert!(sig.is_ok());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn sign_transaction(&self, tx: &mut BundlerTx) -> Result<(), BundlerError> {
        tx.sign(self.currency.get_signer()?).await
    }

    /// Sends a signed transaction
    ///
    /// # Examples
    ///
    /// ```
    /// # use irys_sdk::{
    /// #   currency::TokenType,
    /// #   ClientBuilder,
    /// #   currency::arweave::ArweaveBuilder,
    /// #   tags::Tag,
    /// #   error::BuilderError
    /// # };
    /// # use std::{path::PathBuf, str::FromStr};
    /// # use reqwest::Url;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), BuilderError> {
    /// #   let url = Url::parse("https://uploader.irys.xyz").unwrap();
    /// #   let wallet = PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
    /// #   let currency = ArweaveBuilder::new()
    /// #       .keypair_path(wallet)
    /// #       .build()
    /// #       .expect("Could not create currency instance");
    /// #   let mut bundler_client = ClientBuilder::new()
    /// #       .url(url)
    /// #       .currency(currency)
    /// #       .fetch_pub_info()
    /// #       .await?
    /// #       .build()?;
    /// let data = b"Hello".to_vec();
    /// # let data = b"Hello".to_vec();
    /// # let tags = vec![Tag::new("name", "value")];
    /// let mut tx = bundler_client.create_transaction(data, tags).unwrap();
    /// let sig = bundler_client.sign_transaction(&mut tx).await;
    /// assert!(sig.is_ok());
    /// let result = bundler_client.send_transaction(tx).await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_transaction(&self, tx: BundlerTx) -> Result<UploadReponse, BundlerError> {
        let tx = tx.as_bytes()?;

        let response = self
            .client
            .post(
                self.url
                    .join(&format!("tx/{}", self.currency.get_type()))
                    .map_err(|err| BundlerError::ParseError(err.to_string()))?,
            )
            .header("Content-Type", "application/octet-stream")
            .body(tx)
            .send()
            .await;

        let checked_res = check_and_return::<Value>(response).await?;
        serde_json::from_value(checked_res).map_err(|e| BundlerError::Unknown(e.to_string()))
    }

    /// Sends determined amount to fund an account in the Irys bundler node
    /// # Example
    ///
    /// ```
    /// # use irys_sdk::{
    /// #   currency::TokenType,
    /// #   ClientBuilder,
    /// #   currency::arweave::ArweaveBuilder,
    /// #   tags::Tag,
    /// #   error::BuilderError
    /// # };
    /// # use reqwest::Url;
    /// # use std::{path::PathBuf, str::FromStr};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), BuilderError> {
    /// #   let url = Url::parse("https://uploader.irys.xyz").unwrap();
    /// #   let wallet = PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
    /// #   let currency = ArweaveBuilder::new()
    /// #       .keypair_path(wallet)
    /// #       .build()
    /// #       .expect("Could not create currency instance");
    /// #   let bundler_client = ClientBuilder::new()
    /// #       .url(url)
    /// #       .currency(currency)
    /// #       .fetch_pub_info()
    /// #       .await?
    /// #       .build()?;
    /// let data = b"Hello".to_vec();
    /// let res = bundler_client.fund(data.len() as u64, None).await;
    /// # Ok(())
    /// # }
    pub async fn fund(&self, amount: u64, multiplier: Option<f64>) -> Result<bool, BundlerError> {
        let multiplier = multiplier.unwrap_or(1.0);
        let curr_str = &self.currency.get_type().to_string().to_lowercase();
        let to = match self.pub_info.addresses.get(curr_str) {
            Some(ok) => ok,
            None => return Err(BundlerError::InvalidKey("No address found".to_owned())),
        };
        let fee: u64 = match self.currency.needs_fee() {
            true => self.currency.get_fee(amount, to, multiplier).await?,
            false => Zero::zero(),
        };

        let tx = self.currency.create_tx(amount, to, fee).await?;
        let tx_res = self.currency.send_tx(tx).await?;

        let post_tx_res = self
            .client
            .post(
                self.url
                    .join(&format!("account/balance/{}", self.currency.get_type()))
                    .map_err(|err| BundlerError::ParseError(err.to_string()))?,
            )
            .json(&FundBody {
                tx_id: tx_res.tx_id,
            })
            .send()
            .await;

        check_and_return::<String>(post_tx_res).await.map(|_| true)
    }

    /// Sends a request for withdrawing an amount from Irys bundler node
    /// # Example
    ///
    /// ```
    /// # use irys_sdk::{
    /// #   currency::TokenType,
    /// #   ClientBuilder,
    /// #   currency::arweave::ArweaveBuilder,
    /// #   tags::Tag,
    /// #   error::BuilderError
    /// # };
    /// # use reqwest::Url;
    /// # use std::{path::PathBuf, str::FromStr};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), BuilderError> {
    /// #   let url = Url::parse("https://uploader.irys.xyz").unwrap();
    /// #   let wallet = PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
    /// #   let currency = ArweaveBuilder::new()
    /// #       .keypair_path(wallet)
    /// #       .build()
    /// #       .expect("Could not create currency instance");
    /// #   let bundler_client = ClientBuilder::new()
    /// #       .url(url)
    /// #       .currency(currency)
    /// #       .fetch_pub_info()
    /// #       .await?
    /// #       .build()?;
    /// let res = bundler_client.withdraw(10000).await;
    /// # Ok(())
    /// # }
    pub async fn withdraw(&self, amount: u64) -> Result<bool, BundlerError> {
        let currency_type = self.currency.get_type().to_string().to_lowercase();
        let public_key = Base64(self.currency.get_pub_key()?.to_vec());
        let wallet_address = self.currency.wallet_address()?;
        let nonce = get_nonce(
            &self.client,
            &self.url,
            wallet_address,
            currency_type.clone(),
        )
        .await?;

        let data = DeepHashChunk::Chunks(vec![
            DeepHashChunk::Chunk(Bytes::copy_from_slice(currency_type.as_bytes())),
            DeepHashChunk::Chunk(Bytes::copy_from_slice(amount.to_string().as_bytes())),
            DeepHashChunk::Chunk(Bytes::copy_from_slice(nonce.to_string().as_bytes())),
        ]);

        let dh = deep_hash(data).await?;
        let signature = Base64(self.currency.sign_message(&dh)?);
        self.currency.verify(&public_key.0, &dh, &signature.0)?;

        let data = WithdrawBody {
            public_key: Base64(public_key.to_string().into_bytes()),
            currency: self.currency.get_type().to_string().to_lowercase(),
            amount: amount.to_string(),
            nonce,
            signature: Base64(signature.to_string().into_bytes()),
            sig_type: self.currency.get_type() as u16,
        };

        let res = self
            .client
            .post(
                self.url
                    .join("/account/withdraw")
                    .map_err(|err| BundlerError::ParseError(err.to_string()))?,
            )
            .json(&data)
            .send()
            .await;

        check_and_return::<String>(res).await.map(|_| true)
    }

    /// Upload file on specified path
    ///
    /// # Example
    ///
    /// ```
    /// # use irys_sdk::{
    /// #   currency::TokenType,
    /// #   ClientBuilder,
    /// #   currency::arweave::ArweaveBuilder,
    /// #   tags::Tag,
    /// #   error::BuilderError
    /// # };
    /// # use reqwest::Url;
    /// # use std::{path::PathBuf, str::FromStr};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), BuilderError> {
    /// #   let url = Url::parse("https://uploader.irys.xyz").unwrap();
    /// #   let wallet = PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
    /// #   let currency = ArweaveBuilder::new()
    /// #       .keypair_path(wallet)
    /// #       .build()
    /// #       .expect("Could not create currency instance");
    /// #   let mut bundler_client = ClientBuilder::new()
    /// #       .url(url)
    /// #       .currency(currency)
    /// #       .fetch_pub_info()
    /// #       .await?
    /// #       .build()?;
    /// let file = PathBuf::from_str("res/test_image.jpg").expect("Invalid wallet path");
    /// let result = bundler_client.upload_file(file).await;
    /// #   Ok(())
    /// # }
    /// ```
    pub async fn upload_file(&mut self, file_path: PathBuf) -> Result<UploadReponse, BundlerError> {
        let mut tags = vec![];
        if let Some(content_type) = mime_guess::from_path(file_path.clone()).first() {
            let content_tag: Tag = Tag::new("Content-Type", content_type.as_ref());
            tags.push(content_tag);
        }

        let data = fs::read(&file_path)?;

        // self.uploader.upload(data).await
        let mut tx = self.create_transaction(data, tags)?;
        self.sign_transaction(&mut tx).await?;

        self.send_transaction(tx).await
    }

    /*
    pub async fn upload_directory(
        &self,
        directory_path: PathBuf,
        manifest_path: PathBuf,
    ) -> Result<(), BundlerError> {
        todo!();
    }
    */
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{
        bundler::{get_balance, get_price},
        currency::TokenType,
    };
    use httpmock::{Method::GET, MockServer};
    use num::BigUint;
    use reqwest::Url;

    #[tokio::test]
    async fn should_send_transactions_correctly() {
        //TODO: fix this test
        /*
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST).path("/tx/arweave");
            then.status(200)
                .header("Content-Type", "application/octet-stream")
                .body("{}");
        });
        let mock_2 = server.mock(|when, then| {
            when.method(GET)
                .path("/info");
            then.status(200)
                .body("{ \"version\": \"0\", \"gateway\": \"gateway\", \"addresses\": { \"arweave\": \"address\" }}");
        });

        let url = Url::from_str(&server.url("")).unwrap();
        let path = PathBuf::from_str("res/test_wallet.json").unwrap();
        let currency = Arweave::new(path, url.clone());
        let bundler = &bundler::new(url, &currency).await;
        let tx = bundler.create_transaction_with_tags(
            Vec::from("hello"),
            vec![Tag::new("name".to_string(), "value".to_string())],
        );
        let value = bundler.send_transaction(tx).await.unwrap();

        mock.assert();
        mock_2.assert();
        assert_eq!(value.to_string(), "{}");
        */
    }

    #[tokio::test]
    async fn should_fetch_balance_correctly() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET)
                .path("/account/balance/arweave")
                .query_param("address", "address");
            then.status(200)
                .header("content-type", "application/json")
                .body("{ \"balance\": \"10\" }");
        });

        let url = Url::from_str(&server.url("")).unwrap();
        let address = "address";

        let balance = get_balance(&url, TokenType::Arweave, address, &reqwest::Client::new())
            .await
            .unwrap();

        mock.assert();
        assert_eq!(balance, "10".parse::<BigUint>().unwrap());
    }

    #[tokio::test]
    async fn should_fetch_price_correctly() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(GET).path("/price/arweave/123123123");
            then.status(200)
                .header("content-type", "application/json")
                .body("321321321");
        });

        let url = Url::from_str(&server.url("")).unwrap();
        let balance = get_price(&url, TokenType::Arweave, &reqwest::Client::new(), 123123123)
            .await
            .expect("Could not get price");

        mock.assert();
        assert_eq!(balance, "321321321".parse::<BigUint>().unwrap());
    }

    #[tokio::test]
    async fn should_fund_address_correctly() {}
}
