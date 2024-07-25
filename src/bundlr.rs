use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::{fs, io};

use crate::consts::BUNDLR_DEFAULT_URL;
use crate::currency;
use crate::currency::CurrencyType;
use crate::deep_hash::{deep_hash, DeepHashChunk};
use crate::error::{BuilderError, BundlrError};
use crate::tags::Tag;
use crate::upload::Uploader;
use crate::utils::{check_and_return, get_nonce};
use crate::BundlrTx;
use arweave_rs::crypto::base64::Base64;
use bytes::Bytes;
use num::BigUint;
use num::FromPrimitive;
use num_traits::Zero;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::task;

#[allow(unused)]
pub struct Bundlr<Currency> {
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

pub struct BundlrBuilder<Currency = ()> {
    url: Option<Url>,
    currency: Currency,
    client: Option<reqwest::Client>,
    pub_info: Option<PubInfo>,
}

impl BundlrBuilder {
    pub fn new() -> BundlrBuilder {
        Default::default()
    }
}

impl<Currency> BundlrBuilder<Currency> {
    pub fn url(mut self, url: Url) -> BundlrBuilder<Currency> {
        self.url = Some(url);
        self
    }

    pub fn client(mut self, client: reqwest::Client) -> BundlrBuilder<Currency> {
        self.client = Some(client);
        self
    }

    pub async fn fetch_pub_info(mut self) -> Result<BundlrBuilder<Currency>, BuilderError> {
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

    pub fn pub_info(mut self, pub_info: PubInfo) -> BundlrBuilder<Currency> {
        self.pub_info = Some(pub_info);
        self
    }
}

impl BundlrBuilder<()> {
    pub fn currency<Currency>(self, currency: Currency) -> BundlrBuilder<Currency>
    where
        Currency: currency::Currency,
    {
        BundlrBuilder {
            currency,
            url: self.url,
            client: self.client,
            pub_info: self.pub_info,
        }
    }
}

impl<Currency> BundlrBuilder<Currency>
where
    Currency: currency::Currency,
{
    pub fn build(self) -> Result<Bundlr<Currency>, BuilderError> {
        let url = self.url.unwrap_or(Url::parse(BUNDLR_DEFAULT_URL).unwrap());

        let client = self.client.unwrap_or_else(reqwest::Client::new);

        let pub_info = match self.pub_info {
            Some(p) => p,
            None => return Err(BuilderError::MissingField("currency".to_owned())),
        };

        let uploader = Uploader::new(url.clone(), client.clone(), self.currency.get_type());

        Ok(Bundlr {
            url,
            currency: self.currency,
            client,
            pub_info,
            uploader,
        })
    }
}

/// Gets the public info from a Bundlr node.
///
/// # Examples
///
/// ```
/// # use bundlr_sdk::bundlr::get_pub_info;
/// # use reqwest::Url;
/// # tokio_test::block_on(async {
/// let url = Url::parse("https://node1.bundlr.network/").unwrap();
/// let res = get_pub_info(&url).await;
/// # });
/// ```
pub async fn get_pub_info(url: &Url) -> Result<PubInfo, BundlrError> {
    let client = reqwest::Client::new();
    let response = client
        .get(
            url.join("info")
                .map_err(|err| BundlrError::ParseError(err.to_string()))?,
        )
        .header("Content-Type", "application/json")
        .send()
        .await;

    check_and_return::<PubInfo>(response).await
}

/// Get balance from address in a Bundlr node
pub async fn get_balance(
    url: &Url,
    currency: CurrencyType,
    address: &str,
    client: &reqwest::Client,
) -> Result<BigUint, BundlrError> {
    let response = client
        .get(
            url.join(&format!(
                "account/balance/{}",
                currency.to_string().to_lowercase()
            ))
            .map_err(|err| BundlrError::ParseError(err.to_string()))?,
        )
        .query(&[("address", address)])
        .header("Content-Type", "application/json")
        .send()
        .await;

    match check_and_return::<BalanceResData>(response).await {
        Ok(d) => match BigUint::from_str(&d.balance) {
            Ok(ok) => Ok(ok),
            Err(err) => Err(BundlrError::TypeParseError(err.to_string())),
        },
        Err(err) => Err(BundlrError::TypeParseError(err.to_string())),
    }
}

/// Get the cost for determined amount of bytes, measured in the currency's base units (i.e Winston for Arweave, or Lamport for Solana)
pub async fn get_price(
    url: &Url,
    currency: CurrencyType,
    client: &reqwest::Client,
    byte_amount: u64,
) -> Result<BigUint, BundlrError> {
    let response = client
        .get(
            url.join(&format!("/price/{}/{}", currency, byte_amount))
                .map_err(|err| BundlrError::ParseError(err.to_string()))?,
        )
        .header("Content-Type", "application/json")
        .send()
        .await;

    match check_and_return::<u64>(response).await {
        Ok(d) => match BigUint::from_u64(d) {
            Some(ok) => Ok(ok),
            None => Err(BundlrError::TypeParseError(
                "Could not parse u64 to BigUInt".to_owned(),
            )),
        },
        Err(err) => Err(err),
    }
}

impl<Currency> Bundlr<Currency>
where
    Currency: currency::Currency,
{
    /// Creates an unsigned transaction for posting.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bundlr_sdk::{
    /// #   currency::CurrencyType,
    /// #   BundlrBuilder,
    /// #   currency::arweave::ArweaveBuilder,
    /// #   tags::Tag,
    /// #   error::BuilderError
    /// # };
    /// # use std::{path::PathBuf, str::FromStr};
    /// # use reqwest::Url;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), BuilderError> {
    /// #   let url = Url::parse("https://node1.bundlr.network").unwrap();
    /// #   let wallet = PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
    /// #   let currency = ArweaveBuilder::new()
    /// #       .keypair_path(wallet)
    /// #       .build()
    /// #       .expect("Could not create currency instance");
    /// #   let mut bundlr = BundlrBuilder::new()
    /// #       .url(url)
    /// #       .currency(currency)
    /// #       .fetch_pub_info()
    /// #       .await?
    /// #       .build()?;
    /// let data = b"Hello".to_vec();
    /// let tags = vec![Tag::new("name", "value")];
    /// let tx = bundlr.create_transaction(data, tags).unwrap();
    /// # Ok(())
    /// # }
    /// ```
    pub fn create_transaction(
        &self,
        data: Vec<u8>,
        additional_tags: Vec<Tag>,
    ) -> Result<BundlrTx, BundlrError> {
        BundlrTx::new(vec![], data, additional_tags)
    }

    /// Signs a transaction
    ///
    /// # Examples
    ///
    /// ```
    /// # use bundlr_sdk::{
    /// #   currency::CurrencyType,
    /// #   BundlrBuilder,
    /// #   currency::arweave::ArweaveBuilder,
    /// #   tags::Tag,
    /// #   error::BuilderError
    /// # };
    /// # use std::{path::PathBuf, str::FromStr};
    /// # use reqwest::Url;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), BuilderError> {
    /// #   let url = Url::parse("https://node1.bundlr.network").unwrap();
    /// #   let wallet = PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
    /// #   let currency = ArweaveBuilder::new()
    /// #      .keypair_path(wallet)
    /// #      .build()
    /// #      .expect("Could not create currency instance");
    /// #   let mut bundlr = BundlrBuilder::new()
    /// #       .url(url)
    /// #       .currency(currency)
    /// #       .fetch_pub_info()
    /// #       .await?
    /// #       .build()?;
    /// let data = b"Hello".to_vec();
    /// # let data = b"Hello".to_vec();
    /// # let tags = vec![Tag::new("name", "value")];
    /// let mut tx = bundlr.create_transaction(data, tags).unwrap();
    /// let sig = bundlr.sign_transaction(&mut tx).await;
    /// # assert!(sig.is_ok());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn sign_transaction(&self, tx: &mut BundlrTx) -> Result<(), BundlrError> {
        tx.sign(self.currency.get_signer()?).await
    }

    /// Sends a signed transaction
    ///
    /// # Examples
    ///
    /// ```
    /// # use bundlr_sdk::{
    /// #   currency::CurrencyType,
    /// #   BundlrBuilder,
    /// #   currency::arweave::ArweaveBuilder,
    /// #   tags::Tag,
    /// #   error::BuilderError
    /// # };
    /// # use std::{path::PathBuf, str::FromStr};
    /// # use reqwest::Url;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), BuilderError> {
    /// #   let url = Url::parse("https://node1.bundlr.network").unwrap();
    /// #   let wallet = PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
    /// #   let currency = ArweaveBuilder::new()
    /// #       .keypair_path(wallet)
    /// #       .build()
    /// #       .expect("Could not create currency instance");
    /// #   let mut bundlr = BundlrBuilder::new()
    /// #       .url(url)
    /// #       .currency(currency)
    /// #       .fetch_pub_info()
    /// #       .await?
    /// #       .build()?;
    /// let data = b"Hello".to_vec();
    /// # let data = b"Hello".to_vec();
    /// # let tags = vec![Tag::new("name", "value")];
    /// let mut tx = bundlr.create_transaction(data, tags).unwrap();
    /// let sig = bundlr.sign_transaction(&mut tx).await;
    /// assert!(sig.is_ok());
    /// let result = bundlr.send_transaction(tx).await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_transaction(&self, tx: BundlrTx) -> Result<Value, BundlrError> {
        let tx = tx.as_bytes()?;

        let response = self
            .client
            .post(
                self.url
                    .join(&format!("tx/{}", self.currency.get_type()))
                    .map_err(|err| BundlrError::ParseError(err.to_string()))?,
            )
            .header("Content-Type", "application/octet-stream")
            .body(tx)
            .send()
            .await;

        check_and_return::<Value>(response).await
    }

    /// Sends determined amount to fund an account in the Bundlr node
    /// # Example
    ///
    /// ```
    /// # use bundlr_sdk::{
    /// #   currency::CurrencyType,
    /// #   BundlrBuilder,
    /// #   currency::arweave::ArweaveBuilder,
    /// #   tags::Tag,
    /// #   error::BuilderError
    /// # };
    /// # use reqwest::Url;
    /// # use std::{path::PathBuf, str::FromStr};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), BuilderError> {
    /// #   let url = Url::parse("https://node1.bundlr.network").unwrap();
    /// #   let wallet = PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
    /// #   let currency = ArweaveBuilder::new()
    /// #       .keypair_path(wallet)
    /// #       .build()
    /// #       .expect("Could not create currency instance");
    /// #   let bundlr = BundlrBuilder::new()
    /// #       .url(url)
    /// #       .currency(currency)
    /// #       .fetch_pub_info()
    /// #       .await?
    /// #       .build()?;
    /// let data = b"Hello".to_vec();
    /// let res = bundlr.fund(data.len() as u64, None).await;
    /// # Ok(())
    /// # }
    pub async fn fund(&self, amount: u64, multiplier: Option<f64>) -> Result<bool, BundlrError> {
        let multiplier = multiplier.unwrap_or(1.0);
        let curr_str = &self.currency.get_type().to_string().to_lowercase();
        let to = match self.pub_info.addresses.get(curr_str) {
            Some(ok) => ok,
            None => return Err(BundlrError::InvalidKey("No address found".to_owned())),
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
                    .map_err(|err| BundlrError::ParseError(err.to_string()))?,
            )
            .json(&FundBody {
                tx_id: tx_res.tx_id,
            })
            .send()
            .await;

        check_and_return::<String>(post_tx_res).await.map(|_| true)
    }

    /// Sends a request for withdrawing an amount from Bundlr node
    /// # Example
    ///
    /// ```
    /// # use bundlr_sdk::{
    /// #   currency::CurrencyType,
    /// #   BundlrBuilder,
    /// #   currency::arweave::ArweaveBuilder,
    /// #   tags::Tag,
    /// #   error::BuilderError
    /// # };
    /// # use reqwest::Url;
    /// # use std::{path::PathBuf, str::FromStr};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), BuilderError> {
    /// #   let url = Url::parse("https://node1.bundlr.network").unwrap();
    /// #   let wallet = PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
    /// #   let currency = ArweaveBuilder::new()
    /// #       .keypair_path(wallet)
    /// #       .build()
    /// #       .expect("Could not create currency instance");
    /// #   let bundlr = BundlrBuilder::new()
    /// #       .url(url)
    /// #       .currency(currency)
    /// #       .fetch_pub_info()
    /// #       .await?
    /// #       .build()?;
    /// let res = bundlr.withdraw(10000).await;
    /// # Ok(())
    /// # }
    pub async fn withdraw(&self, amount: u64) -> Result<bool, BundlrError> {
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
                    .map_err(|err| BundlrError::ParseError(err.to_string()))?,
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
    /// # use bundlr_sdk::{
    /// #   currency::CurrencyType,
    /// #   BundlrBuilder,
    /// #   currency::arweave::ArweaveBuilder,
    /// #   tags::Tag,
    /// #   error::BuilderError
    /// # };
    /// # use reqwest::Url;
    /// # use std::{path::PathBuf, str::FromStr};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), BuilderError> {
    /// #   let url = Url::parse("https://node1.bundlr.network").unwrap();
    /// #   let wallet = PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
    /// #   let currency = ArweaveBuilder::new()
    /// #       .keypair_path(wallet)
    /// #       .build()
    /// #       .expect("Could not create currency instance");
    /// #   let mut bundlr = BundlrBuilder::new()
    /// #       .url(url)
    /// #       .currency(currency)
    /// #       .fetch_pub_info()
    /// #       .await?
    /// #       .build()?;
    /// let file = PathBuf::from_str("res/test_image.jpg").expect("Invalid wallet path");
    /// let result = bundlr.upload_file(file).await;
    /// #   Ok(())
    /// # }
    /// ```
    pub async fn upload_file(&mut self, file_path: PathBuf) -> Result<(), BundlrError> {
        let mut tags = vec![];
        if let Some(content_type) = mime_guess::from_path(file_path.clone()).first() {
            let content_tag: Tag = Tag::new("Content-Type", content_type.as_ref());
            tags.push(content_tag);
        }

        let data = fs::read(&file_path)?;

        self.uploader.upload(data).await
    }

    pub async fn upload_directory(
        &mut self,
        directory_path: PathBuf,
        //  manifest_path: PathBuf,
    ) -> Result<(), BundlrError> {
        let entries = task::block_in_place(|| {
            fs::read_dir(&directory_path).map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            let path = entry.path();
            let mut uploader = self.uploader.clone();
            if path.is_dir() {
                println!("Is Dir");
                let file_data =
                    task::block_in_place(|| fs::read(&path).map_err(|e| eprintln!("error[{}]", e)));
                uploader.upload(file_data.unwrap()).await?;
            } else if path.is_file() {
                println!("Is File");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[cfg(feature = "skip_for_now")]

mod tests {
    use std::str::FromStr;

    use crate::{
        bundlr::{get_balance, get_price},
        currency::CurrencyType,
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
        let bundler = &Bundlr::new(url, &currency).await;
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

        let balance = get_balance(
            &url,
            CurrencyType::Arweave,
            address,
            &reqwest::Client::new(),
        )
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
        let balance = get_price(
            &url,
            CurrencyType::Arweave,
            &reqwest::Client::new(),
            123123123,
        )
        .await
        .expect("Could not get price");

        mock.assert();
        assert_eq!(balance, "321321321".parse::<BigUint>().unwrap());
    }

    #[tokio::test]
    async fn should_upload_dir() {
        let url = Url::parse("https://node1.bundlr.network").unwrap();
        let wallet =
            std::path::PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
        let _currency = arweave_rs::ArweaveBuilder::new()
            .keypair_path(wallet)
            .build()
            .expect("Could not create currency instance");
        let mut _bundlr = crate::BundlrBuilder::new()
            .url(url)
            .currency(currency) // <- issue with trait Currency::currency
            .fetch_pub_info()
            .await
            .unwrap()
            .build()?; // <- method not found

        let _dir = std::path::PathBuf::from_str("res").expect("Invalid dir path");
        let _result = todo!();
        bundlr.upload_dir(dir).await;
    }

    #[tokio::test]
    async fn should_fund_address_correctly() {}
}
