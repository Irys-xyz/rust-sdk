use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use crate::currency::Currency;
use crate::currency::CurrencyType;
use crate::deep_hash::{deep_hash, DeepHashChunk};
use crate::error::BundlrError;
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
    /// let currency = Arweave::new(wallet, None).unwrap();
    /// let bundlr = Bundlr::new(url, &currency).await.unwrap();
    /// # })
    /// ```
    pub async fn new(url: Url, currency: &dyn Currency) -> Result<Bundlr, BundlrError> {
        let pub_info = Bundlr::get_pub_info(&url).await?;
        let uploader = Uploader::new(url.clone(), reqwest::Client::new(), currency.get_type());

        Ok(Bundlr {
            url,
            currency,
            client: reqwest::Client::new(),
            pub_info,
            uploader,
        })
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
    /// Bundlr::get_pub_info(&url).await.unwrap();
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
    /// # let currency = Arweave::new(wallet, None).unwrap();
    /// # let bundlr = Bundlr::new(url, &currency).await.unwrap();
    /// let data = b"Hello".to_vec();
    /// let tags = vec![Tag::new("name", "value")];
    /// let tx = bundlr.create_transaction(data, tags).unwrap();
    /// # });
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
    /// # use bundlr_sdk::{Bundlr, currency::arweave::Arweave, tags::Tag};
    /// # use std::{path::PathBuf, str::FromStr};
    /// # use reqwest::Url;
    /// # tokio_test::block_on(async {
    /// # let url = Url::parse("https://node1.bundlr.network").unwrap();
    /// # let wallet = PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
    /// # let currency = Arweave::new(wallet, None).unwrap();
    /// # let bundlr = Bundlr::new(url, &currency).await.unwrap();
    /// # let data = b"Hello".to_vec();
    /// # let tags = vec![Tag::new("name", "value")];
    /// let mut tx = bundlr.create_transaction(data, tags).unwrap();
    /// let sig = bundlr.sign_transaction(&mut tx).await;
    /// assert!(sig.is_ok());
    /// # });
    /// ```
    pub async fn sign_transaction(&self, tx: &mut BundlrTx) -> Result<(), BundlrError> {
        tx.sign(self.currency.get_signer()?).await
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
    /// # let currency = Arweave::new(wallet, None).unwrap();
    /// # let bundlr = Bundlr::new(url, &currency).await.unwrap();
    /// # let data = b"Hello".to_vec();
    /// # let tags = vec![Tag::new("name", "value")];
    /// let mut tx = bundlr.create_transaction(data, tags).unwrap();
    /// let sig = bundlr.sign_transaction(&mut tx).await;
    /// assert!(sig.is_ok());
    /// let result = bundlr.send_transaction(tx).await;
    /// # });
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
    /// let currency = Arweave::new(wallet, None).unwrap();
    /// let bundlr = Bundlr::new(url, &currency).await.unwrap();
    /// let res = bundlr.get_balance("address").await;
    /// assert!(res.is_ok());
    /// # })
    pub async fn get_balance(&self, address: &str) -> Result<BigUint, BundlrError> {
        Bundlr::get_balance_public(&self.url, self.currency.get_type(), address, &self.client).await
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
    /// let currency = Arweave::new(wallet, None).unwrap();
    /// let bundlr = Bundlr::new(url, &currency).await.unwrap();
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
    /// let currency = Arweave::new(wallet, None).unwrap();
    /// let bundlr = Bundlr::new(url, &currency).await.unwrap();
    /// let res = bundlr.fund(10000, None)/*.await*/;
    /// # })
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
    /// # use bundlr_sdk::{currency::CurrencyType, Bundlr, currency::arweave::Arweave};
    /// # use reqwest::Url;
    /// # use std::{path::PathBuf, str::FromStr};
    /// # tokio_test::block_on(async {
    /// # let url = Url::parse("https://node1.bundlr.network").unwrap();
    /// # let wallet = PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
    /// let currency = Arweave::new(wallet, None).unwrap();
    /// let bundlr = Bundlr::new(url, &currency).await.unwrap();
    /// let res = bundlr.withdraw(10000).await;
    /// # })
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
    /// # use bundlr_sdk::{currency::CurrencyType, Bundlr, currency::arweave::Arweave};
    /// # use reqwest::Url;
    /// # use std::{path::PathBuf, str::FromStr};
    /// # tokio_test::block_on(async {
    /// # let url = Url::parse("https://node1.bundlr.network").unwrap();
    /// # let wallet = PathBuf::from_str("res/test_wallet.json").expect("Invalid wallet path");
    /// let currency = Arweave::new(wallet, None).unwrap();
    /// let mut bundlr = Bundlr::new(url, &currency).await.unwrap();
    /// let file = PathBuf::from_str("res/test_image.jpg").expect("Invalid wallet path");
    /// let result = bundlr.upload_file(file).await;
    /// # })
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

    /*
    pub async fn upload_directory(
        &self,
        directory_path: PathBuf,
        manifest_path: PathBuf,
    ) -> Result<(), BundlrError> {
        todo!();
    }
    */
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
        let currency = Arweave::new(path, Some(url.clone())).unwrap();
        let bundler = &Bundlr::new(url, &currency).await.unwrap();
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
        let currency = Arweave::new(path, Some(url.clone())).unwrap();
        let bundler = &Bundlr::new(url, &currency).await.unwrap();
        let balance = bundler.get_price(123123123).await.expect("wtf");

        mock.assert();
        mock_2.assert();
        assert_eq!(balance, "321321321".parse::<BigUint>().unwrap());
    }

    #[tokio::test]
    async fn should_fund_address_correctly() {}
}
