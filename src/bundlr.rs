use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use crate::currency::CurrencyType;
use crate::deep_hash::{deep_hash, DeepHashChunk};
use crate::error::BundlrError;
use crate::tags::Tag;
use crate::upload::Uploader;
use crate::utils::{check_and_return, get_nonce};
use crate::BundlrTx;
use crate::{currency::Currency, transaction::poll::ConfirmationPoll};
use arweave_rs::crypto::base64::Base64;
use bytes::Bytes;
use num::{BigUint, FromPrimitive};
use num_traits::Zero;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[allow(unused)]
pub struct Bundlr<'a> {
    url: Url,
    currency: &'a dyn Currency,
    client: reqwest::Client,
    pub_info: PubInfo,
    uploader: Uploader,
}
#[derive(Deserialize, Default)]
pub struct BalanceResData {
    balance: String,
}
#[allow(unused)]
#[derive(Deserialize, Default)]
pub struct PubInfo {
    version: String,
    gateway: String,
    addresses: HashMap<String, String>,
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

impl Bundlr<'_> {
    /// Creates a new Bundlr client, based in a currency.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bundlr_sdk::{Bundlr, currency::arweave::Arweave};
    /// # use std::{path::PathBuf, str::FromStr};
    /// # use reqwest::Url;
    /// # tokio_test::block_on(async {
    /// let url = Url::parse("https://node1.bundlr.network").unwrap();
    /// let wallet = PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
    /// let currency = Arweave::new(wallet, None);
    /// let bundlr = Bundlr::new(url, &currency).await;
    /// # })
    /// ```
    pub async fn new(url: Url, currency: &dyn Currency) -> Bundlr {
        let pub_info = Bundlr::get_pub_info(&url)
            .await
            .unwrap_or_else(|_| panic!("Could not fetch public info from url: {}", url));
        let uploader = Uploader::new(url.clone(), reqwest::Client::new(), currency.get_type());

        Bundlr {
            url,
            currency,
            client: reqwest::Client::new(),
            pub_info,
            uploader,
        }
    }

    /// Gets the public info from a Bundlr node.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bundlr_sdk::Bundlr;
    /// # use reqwest::Url;
    /// # tokio_test::block_on(async {
    /// let url = Url::parse("https://node1.bundlr.network/").unwrap();
    /// Bundlr::get_pub_info(&url).await;
    /// # });
    /// ```
    pub async fn get_pub_info(url: &Url) -> Result<PubInfo, BundlrError> {
        let client = reqwest::Client::new();
        let response = client
            .get(url.join("info").expect("Could not join url with /info"))
            .header("Content-Type", "application/json")
            .send()
            .await;

        check_and_return::<PubInfo>(response).await
    }

    /// Creates an unsigned transaction for posting.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bundlr_sdk::{Bundlr, currency::arweave::Arweave, tags::Tag};
    /// # use std::{path::PathBuf, str::FromStr};
    /// # use reqwest::Url;
    /// # tokio_test::block_on(async {
    /// # let url = Url::parse("https://node1.bundlr.network").unwrap();
    /// # let wallet = PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
    /// # let currency = Arweave::new(wallet, None);
    /// # let bundlr = Bundlr::new(url, &currency).await;
    /// let data = b"Hello".to_vec();
    /// let tags = vec![Tag::new("name", "value")];
    /// let tx = bundlr.create_transaction(data, tags);
    /// # });
    /// ```
    pub fn create_transaction(&self, data: Vec<u8>, additional_tags: Vec<Tag>) -> BundlrTx {
        BundlrTx::new(vec![], data, additional_tags)
    }

    /// Signs a transaction
    ///
    /// # Examples
    ///
    /// ```
    /// # use bundlr_sdk::{Bundlr, currency::arweave::Arweave, tags::Tag};
    /// # use std::{path::PathBuf, str::FromStr};
    /// # use reqwest::Url;
    /// # tokio_test::block_on(async {
    /// # let url = Url::parse("https://node1.bundlr.network").unwrap();
    /// # let wallet = PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
    /// # let currency = Arweave::new(wallet, None);
    /// # let bundlr = Bundlr::new(url, &currency).await;
    /// # let data = b"Hello".to_vec();
    /// # let tags = vec![Tag::new("name", "value")];
    /// let mut tx = bundlr.create_transaction(data, tags);
    /// let sig = bundlr.sign_transaction(&mut tx).await;
    /// assert!(sig.is_ok());
    /// # });
    /// ```
    pub async fn sign_transaction(&self, tx: &mut BundlrTx) -> Result<(), BundlrError> {
        tx.sign(self.currency.get_signer()).await
    }

    /// Sends a signed transaction
    ///
    /// # Examples
    ///
    /// ```
    /// # use bundlr_sdk::{Bundlr, currency::arweave::Arweave, tags::Tag};
    /// # use std::{path::PathBuf, str::FromStr};
    /// # use reqwest::Url;
    /// # tokio_test::block_on(async {
    /// # let url = Url::parse("https://node1.bundlr.network").unwrap();
    /// # let wallet = PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
    /// # let currency = Arweave::new(wallet, None);
    /// # let bundlr = Bundlr::new(url, &currency).await;
    /// # let data = b"Hello".to_vec();
    /// # let tags = vec![Tag::new("name", "value")];
    /// let mut tx = bundlr.create_transaction(data, tags);
    /// let sig = bundlr.sign_transaction(&mut tx).await;
    /// assert!(sig.is_ok());
    /// let result = bundlr.send_transaction(tx).await;
    /// # });
    /// ```
    pub async fn send_transaction(&self, tx: BundlrTx) -> Result<Value, BundlrError> {
        let tx = tx.as_bytes().expect("Could not serialize transaction");

        let response = self
            .client
            .post(
                self.url
                    .join(&format!("tx/{}", self.currency.get_type()))
                    .expect("Could not join url with /tx/{}"),
            )
            .header("Content-Type", "application/octet-stream")
            .body(tx)
            .send()
            .await;

        check_and_return::<Value>(response).await
    }

    /// Get balance from address in a Bundlr node
    /// # Example
    ///
    /// ```
    /// # use bundlr_sdk::{currency::CurrencyType, Bundlr};
    /// # use reqwest::Url;
    /// # tokio_test::block_on(async {
    /// let url = Url::parse("https://node1.bundlr.network").unwrap();
    /// let currency = CurrencyType::Solana;
    /// let address = "address";
    /// let res = Bundlr::get_balance_public(&url, currency, &address, &reqwest::Client::new()).await;
    /// assert!(res.is_ok());
    /// # })
    pub async fn get_balance_public(
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
                .expect("Could not join url with /account/balance/{}"),
            )
            .query(&[("address", address)])
            .header("Content-Type", "application/json")
            .send()
            .await;

        check_and_return::<BalanceResData>(response)
            .await
            .map(|d| BigUint::from_str(&d.balance).expect("Error converting from u128 to BigUint"))
    }

    /// Get balance from address in a Bundlr node
    /// # Example
    ///
    /// ```
    /// # use bundlr_sdk::{currency::CurrencyType, Bundlr, currency::arweave::Arweave};
    /// # use reqwest::Url;
    /// # use std::{path::PathBuf, str::FromStr};
    /// # tokio_test::block_on(async {
    /// let url = Url::parse("https://node1.bundlr.network").unwrap();
    /// # let wallet = PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
    /// let currency = Arweave::new(wallet, None);
    /// let bundlr = Bundlr::new(url, &currency).await;
    /// let res = bundlr.get_balance("address").await;
    /// assert!(res.is_ok());
    /// # })
    pub async fn get_balance(&self, address: &str) -> Result<BigUint, BundlrError> {
        Bundlr::get_balance_public(&self.url, self.currency.get_type(), &address, &self.client)
            .await
    }

    /// Get the cost for determined amount of bytes, measured in the currency's base unit
    /// # Example
    ///
    /// ```
    /// # use bundlr_sdk::{currency::CurrencyType, Bundlr, currency::arweave::Arweave};
    /// # use reqwest::Url;
    /// # use std::{path::PathBuf, str::FromStr};
    /// # tokio_test::block_on(async {
    /// let url = Url::parse("https://node1.bundlr.network").unwrap();
    /// # let wallet = PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
    /// let currency = Arweave::new(wallet, None);
    /// let bundlr = Bundlr::new(url, &currency).await;
    /// let res = bundlr.get_price(2560000).await;
    /// assert!(res.is_ok());
    /// # })
    pub async fn get_price(&self, byte_amount: u64) -> Result<BigUint, BundlrError> {
        Bundlr::get_price_public(
            &self.url,
            self.currency.get_type(),
            &self.client,
            byte_amount,
        )
        .await
    }

    /// Get the cost for determined amount of bytes, measured in the currency's base unit
    /// # Example
    ///
    /// ```
    /// # use bundlr_sdk::{currency::CurrencyType, Bundlr};
    /// # use reqwest::Url;
    /// # tokio_test::block_on(async {
    /// let url = Url::parse("https://node1.bundlr.network").unwrap();
    /// let currency = CurrencyType::Solana;
    /// let address = "address";
    /// let res = Bundlr::get_price_public(&url, currency, &reqwest::Client::new(), 256000).await;
    /// assert!(res.is_ok());
    /// # })
    pub async fn get_price_public(
        url: &Url,
        currency: CurrencyType,
        client: &reqwest::Client,
        byte_amount: u64,
    ) -> Result<BigUint, BundlrError> {
        let response = client
            .get(
                url.join(&format!("/price/{}/{}", currency, byte_amount))
                    .expect("Could not join url with /price/{}/{}"),
            )
            .header("Content-Type", "application/json")
            .send()
            .await;

        check_and_return::<u64>(response)
            .await
            .map(|d| BigUint::from_u64(d).expect("Error converting from string to BigUint"))
    }

    /// Sends determined amount to fund an account in the Bundlr node
    /// # Example
    ///
    /// ```
    /// # use bundlr_sdk::{currency::CurrencyType, Bundlr, currency::arweave::Arweave};
    /// # use reqwest::Url;
    /// # use std::{path::PathBuf, str::FromStr};
    /// # tokio_test::block_on(async {
    /// # let url = Url::parse("https://node1.bundlr.network").unwrap();
    /// # let wallet = PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
    /// let currency = Arweave::new(wallet, None);
    /// let bundlr = Bundlr::new(url, &currency).await;
    /// let res = bundlr.fund(10000, None)/*.await*/;
    /// # })
    pub async fn fund(&self, amount: u64, multiplier: Option<f64>) -> Result<bool, BundlrError> {
        let multiplier = multiplier.unwrap_or(1.0);
        let curr_str = &self.currency.get_type().to_string().to_lowercase();
        let to = self
            .pub_info
            .addresses
            .get(curr_str)
            .expect("Address should not be empty");
        let fee: u64 = match self.currency.needs_fee() {
            true => self.currency.get_fee(amount, to, multiplier).await,
            false => Zero::zero(),
        };

        let tx = self.currency.create_tx(amount, to, fee).await;
        let tx_res = self
            .currency
            .send_tx(tx)
            .await
            .expect("Error while sending transaction");

        let post_tx_res = self
            .client
            .post(
                self.url
                    .join(&format!("account/balance/{}", self.currency.get_type()))
                    .expect("Could not join url with /account/balance/{}"),
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
    /// # use bundlr_sdk::{currency::CurrencyType, Bundlr, currency::arweave::Arweave};
    /// # use reqwest::Url;
    /// # use std::{path::PathBuf, str::FromStr};
    /// # tokio_test::block_on(async {
    /// # let url = Url::parse("https://node1.bundlr.network").unwrap();
    /// # let wallet = PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
    /// let currency = Arweave::new(wallet, None);
    /// let bundlr = Bundlr::new(url, &currency).await;
    /// let res = bundlr.withdraw(10000).await;
    /// # })
    pub async fn withdraw(&self, amount: u64) -> Result<bool, BundlrError> {
        let currency_type = self.currency.get_type().to_string().to_lowercase();
        let public_key = Base64(self.currency.get_pub_key().to_vec());
        let wallet_address = self.currency.wallet_address();
        let nonce = get_nonce(
            &self.client,
            &self.url,
            wallet_address,
            currency_type.clone(),
        )
        .await
        .expect("Could not get nonce");

        let data = DeepHashChunk::Chunks(vec![
            DeepHashChunk::Chunk(Bytes::copy_from_slice(currency_type.as_bytes())),
            DeepHashChunk::Chunk(Bytes::copy_from_slice(amount.to_string().as_bytes())),
            DeepHashChunk::Chunk(Bytes::copy_from_slice(&nonce.to_string().as_bytes())),
        ]);

        let dh = deep_hash(data).await.expect("Could not deep hash item");
        let signature = Base64(self.currency.sign_message(&dh));
        self.currency
            .verify(&public_key.0, &dh, &signature.0)
            .expect("Signature not ok");

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
                    .expect("Could not join url with /account/withdraw"),
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
    /// # use bundlr_sdk::{currency::CurrencyType, Bundlr, currency::arweave::Arweave};
    /// # use reqwest::Url;
    /// # use std::{path::PathBuf, str::FromStr};
    /// # tokio_test::block_on(async {
    /// # let url = Url::parse("https://node1.bundlr.network").unwrap();
    /// # let wallet = PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
    /// let currency = Arweave::new(wallet, None);
    /// let mut bundlr = Bundlr::new(url, &currency).await;
    /// let file = PathBuf::from_str("res/test_image.jpg").expect("Invalid wallet path");
    /// let result = bundlr.upload_file(file).await;
    /// # })
    /// ```
    pub async fn upload_file(&mut self, file_path: PathBuf) -> Result<(), BundlrError> {
        let mut tags = vec![];
        if let Some(content_type) = mime_guess::from_path(file_path.clone()).first() {
            let content_tag: Tag = Tag::new("Content-Type", &content_type.to_string());
            tags.push(content_tag);
        }

        let data = fs::read(&file_path).expect("Could not read file");

        self.uploader.upload(data).await
    }

    pub async fn upload_directory(
        &self,
        directory_path: PathBuf,
        manifest_path: PathBuf,
    ) -> Result<(), BundlrError> {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use crate::{currency::arweave::Arweave, Bundlr};
    use httpmock::{Method::GET, MockServer};
    use num::BigUint;
    use reqwest::Url;

    #[tokio::test]
    async fn should_send_transactions_correctly() {
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
        let mock_2 = server.mock(|when, then| {
            when.method(GET)
                .path("/info");
            then.status(200)
                .body("{ \"version\": \"0\", \"gateway\": \"gateway\", \"addresses\": { \"arweave\": \"address\" }}");  
        });

        let url = Url::from_str(&server.url("")).unwrap();
        let address = "address";
        let path = PathBuf::from_str("res/test_wallet.json").unwrap();
        println!("{:?}", &path);
        let currency = Arweave::new(path, Some(url.clone()));
        let bundler = &Bundlr::new(url, &currency).await;
        let balance = bundler.get_balance(address).await.unwrap();

        mock.assert();
        mock_2.assert();
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
        let mock_2 = server.mock(|when, then| {
            when.method(GET)
                .path("/info");
            then.status(200)
                .body("{ \"version\": \"0\", \"gateway\": \"gateway\", \"addresses\": { \"arweave\": \"address\" }}");  
        });

        let url = Url::from_str(&server.url("")).unwrap();
        let path = PathBuf::from_str("res/test_wallet.json").unwrap();
        println!("{:?}", &path);
        let currency = Arweave::new(path, Some(url.clone()));
        let bundler = &Bundlr::new(url, &currency).await;
        let balance = bundler.get_price(123123123).await.unwrap();

        mock.assert();
        mock_2.assert();
        assert_eq!(balance, "321321321".parse::<BigUint>().unwrap());
    }

    #[tokio::test]
    async fn should_fund_address_correctly() {}
}
