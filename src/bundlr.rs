use std::collections::HashMap;

use crate::currency::Currency;
use crate::error::BundlrError;
use crate::tags::Tag;
use crate::{signers::Signer, BundlrTx};
use num::{BigRational, BigUint};
use num_traits::Zero;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[allow(unused)]
pub struct Bundlr<'a> {
    url: String, // FIXME: type of this field should be Url
    currency: Currency,
    signer: &'a dyn Signer,
    client: reqwest::Client,
    pub_info: PubInfo,
}

#[allow(unused)]
#[derive(Deserialize)]
pub struct TxResponse {
    id: String,
}

#[derive(Deserialize)]
pub struct BalanceResData {
    balance: String,
}
#[allow(unused)]
#[derive(Deserialize)]
pub struct PubInfo {
    version: String,
    gateway: String,
    addresses: HashMap<String, String>,
}

#[derive(Serialize)]
pub struct FundBody {
    tx_id: String,
}

impl Bundlr<'_> {
    pub async fn new(url: String, currency: Currency, signer: &dyn Signer) -> Bundlr {
        let pub_info = Bundlr::get_pub_info(&url)
            .await
            .unwrap_or_else(|_| panic!("Could not fetch public info from url: {}", url));

        Bundlr {
            url,
            currency,
            signer,
            client: reqwest::Client::new(),
            pub_info,
        }
    }

    pub async fn get_pub_info(url: &String) -> Result<PubInfo, BundlrError> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/info", url))
            .header("Content-Type", "application/json")
            .send()
            .await;
        match response {
            Ok(r) => {
                if !r.status().is_success() {
                    let msg = format!("Status: {}", r.status());
                    return Err(BundlrError::ResponseError(msg));
                };
                r.json::<PubInfo>()
                    .await
                    .map_err(|err| BundlrError::RequestError(err.to_string()))
            }
            Err(err) => Err(BundlrError::ResponseError(err.to_string())),
        }
    }

    pub fn create_transaction_with_tags(&self, data: Vec<u8>, tags: Vec<Tag>) -> BundlrTx {
        BundlrTx::create_with_tags(data, tags, self.signer)
    }

    pub async fn send_transaction(&self, tx: BundlrTx) -> Result<Value, BundlrError> {
        let tx = tx.into_inner();

        let response = self
            .client
            .post(format!("{}/tx/{}", self.url, self.currency))
            .header("Content-Type", "application/octet-stream")
            .body(tx)
            .send()
            .await;

        match response {
            Ok(r) => {
                if !r.status().is_success() {
                    let msg = format!("Status: {}", r.status());
                    return Err(BundlrError::ResponseError(msg));
                };
                r.json::<Value>()
                    .await
                    .map_err(|e| BundlrError::ResponseError(e.to_string()))
            }
            Err(err) => Err(BundlrError::ResponseError(err.to_string())),
        }
    }

    pub async fn get_balance_public(
        url: &str,
        currency: &Currency,
        address: &str,
        client: &reqwest::Client,
    ) -> Result<BigUint, BundlrError> {
        let response = client
            .get(format!("{}/account/balance/{}", url, currency))
            .query(&[("address", address)])
            .header("Content-Type", "application/json")
            .send()
            .await;

        match response {
            Ok(r) => {
                if !r.status().is_success() {
                    let msg = format!("Status: {}", r.status());
                    return Err(BundlrError::ResponseError(msg));
                };
                let data = r.json::<BalanceResData>().await.unwrap();
                data.balance
                    .parse::<BigUint>()
                    .map_err(|err| BundlrError::RequestError(err.to_string()))
            }
            Err(err) => Err(BundlrError::ResponseError(err.to_string())),
        }
    }

    pub async fn get_balance(&self, address: String) -> Result<BigUint, BundlrError> {
        Bundlr::get_balance_public(&self.url, &self.currency, &address, &self.client).await
    }

    pub async fn fund(
        &self,
        amount: BigUint,
        multiplier: Option<BigRational>,
    ) -> Result<bool, BundlrError> {
        let curr_str = &self.currency.to_string().to_lowercase();
        let to = self
            .pub_info
            .addresses
            .get(curr_str)
            .expect("Address should not be empty");
        let fee: BigUint = match self.currency.needs_fee() {
            true => self.currency.get_fee(&amount, to, multiplier).await,
            false => Zero::zero(),
        };
        let _tx = self.currency.create_tx(&amount, to, &fee).await;

        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{currency::Currency, tags::Tag, wallet::load_from_file, ArweaveSigner, Bundlr};
    use httpmock::{
        Method::{GET, POST},
        MockServer,
    };
    use num::BigUint;
    #[tokio::test]
    async fn should_send_transactions_correctly() {
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

        let url = server.url("");
        let currency = Currency::Arweave;
        let jwk =
            load_from_file(&"res/test_wallet.json".to_string()).expect("Error loading wallet");
        let signer = &ArweaveSigner::from_jwk(jwk);
        let bundler = &Bundlr::new(url.to_string(), currency, signer).await;
        let tx = bundler.create_transaction_with_tags(
            Vec::from("hello"),
            vec![Tag::new("name".to_string(), "value".to_string())],
        );
        let value = bundler.send_transaction(tx).await.unwrap();

        mock.assert();
        mock_2.assert();
        assert_eq!(value.to_string(), "{}");
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

        let url = server.url("");
        let address = "address";
        let currency = Currency::Arweave;
        let jwk =
            load_from_file(&"res/test_wallet.json".to_string()).expect("Error loading wallet");
        let signer = &ArweaveSigner::from_jwk(jwk);
        let bundler = &Bundlr::new(url.to_string(), currency, signer).await;
        let balance = bundler.get_balance(address.to_string()).await.unwrap();

        mock.assert();
        mock_2.assert();
        assert_eq!(balance, "10".parse::<BigUint>().unwrap());
    }

    #[tokio::test]
    async fn should_fund_address_correctly() {
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

        let url = server.url("");
        let address = "address";
        let currency = Currency::Arweave;
        let jwk =
            load_from_file(&"res/test_wallet.json".to_string()).expect("Error loading wallet");
        let signer = &ArweaveSigner::from_jwk(jwk);
        let bundler = &Bundlr::new(url.to_string(), currency, signer).await;
        let balance = bundler.get_balance(address.to_string()).await.unwrap();

        mock.assert();
        mock_2.assert();
        assert_eq!(balance, "10".parse::<BigUint>().unwrap());
    }
}
