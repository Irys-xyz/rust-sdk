use serde::Deserialize;
use serde_json::Value;

use crate::currency::Currency;
use crate::error::BundlrError;
use crate::tags::Tag;
use crate::{signers::signer::Signer, BundlrTx};

pub struct Bundlr {
    url: String,
    currency: Currency,
    signer: Option<Box<dyn Signer>>,
    client: reqwest::Client,
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

impl Bundlr {
    pub fn new(url: String, currency: Currency, signer: Option<Box<dyn Signer>>) -> Bundlr {
        Bundlr {
            url,
            currency,
            signer,
            client: reqwest::Client::new(),
        }
    }

    pub fn create_transaction_with_tags(&self, data: Vec<u8>, tags: Vec<Tag>) -> BundlrTx {
        match self.signer.is_some() {
            true => BundlrTx::create_with_tags(data, tags, self.signer.as_ref().unwrap()),
            false => panic!("No secret key present"),
        }
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

    pub async fn get_balance(&self, address: String) -> Result<u64, BundlrError> {
        let response = self
            .client
            .get(format!("{}/account/balance/{}", &self.url, &self.currency))
            .query(&[("address", address.as_str())])
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
                    .parse::<u64>()
                    .map_err(|err| BundlrError::RequestError(err.to_string()))
            }
            Err(err) => Err(BundlrError::ResponseError(err.to_string())),
        }
    }

    pub async fn fund_account(&self) -> Result<Value, BundlrError> {
        /*
            let response = self
                .client
                .post(format!("{}/account/balance/{}", &self.url, &self.currency))
                .query(&[("address", address.as_str())])
                .header("Content-Type", "application/json")
                .send()
                .await;
        */
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use crate::{currency::Currency, tags::Tag, wallet, ArweaveSigner, Bundlr, Signer};
    use httpmock::{
        Method::{GET, POST},
        MockServer,
    };
    use jsonwebkey as jwk;
    use std::str::FromStr;

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

        let url = server.url("");
        let address = "address";
        let currency = Currency::from_str("arweave").unwrap();
        let bundler = &Bundlr::new(url.to_string(), currency, None);
        let balance = bundler.get_balance(address.to_string()).await.unwrap();

        mock.assert();
        assert_eq!(balance, 10);
    }

    #[tokio::test]
    async fn should_send_transactions_correctly() {
        let server = MockServer::start();
        let mock = server.mock(|when, then| {
            when.method(POST).path("/tx/arweave");
            then.status(200)
                .header("Content-Type", "application/octet-stream")
                .body("{}");
        });

        let url = server.url("");
        let currency = Currency::from_str("arweave").unwrap();
        let jwk: jwk::JsonWebKey = wallet::load_from_file("res/test_wallet.json");
        let signer = Box::new(ArweaveSigner::from_jwk(jwk));
        let bundler = &Bundlr::new(url.to_string(), currency, Some(signer));
        let tx = bundler.create_transaction_with_tags(
            Vec::from("hello"),
            vec![Tag::new("name".to_string(), "value".to_string())],
        );
        let value = bundler.send_transaction(tx).await.unwrap();

        mock.assert();
        assert_eq!(value.to_string(), "{}");
    }

    #[test]
    #[should_panic]
    fn should_panic_when_creating_txs_without_secret_key() {
        let currency = Currency::from_str("arweave").unwrap();
        let bundler = &Bundlr::new("".to_string(), currency, None);
        bundler.create_transaction_with_tags(
            Vec::from("hello"),
            vec![Tag::new("name".to_string(), "value".to_string())],
        );
    }
}
