use serde::Deserialize;
use serde_json::Value;

use crate::currency::Currency;
use crate::error::BundlrError;
use crate::tags::Tag;
use crate::{signers::signer::Signer, BundlrTx};

pub struct Bundlr<'a> {
    url: String,
    currency: Currency,
    signer: Option<&'a dyn Signer>,
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

impl Bundlr<'_> {
    pub fn new(url: String, currency: Currency, signer: Option<&dyn Signer>) -> Bundlr {
        Bundlr {
            url,
            currency,
            signer,
            client: reqwest::Client::new(),
        }
    }

    pub fn create_transaction_with_tags(&self, data: Vec<u8>, tags: Vec<Tag>) -> BundlrTx {
        match self.signer.is_some() {
            true => BundlrTx::create_with_tags(data, tags, self.signer.unwrap()),
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
}
