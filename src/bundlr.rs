use std::fs::File;
use std::io::Write;

use serde::Deserialize;
use serde_json::Value;

use crate::error::BundlrError;
use crate::tags::Tag;
use crate::{signers::signer::Signer, BundlrTx};

#[allow(unused)]
pub struct Bundlr<T> {
    url: String,
    chain: String,
    currency: String,
    signer: T,
    client: reqwest::Client,
}

#[allow(unused)]
#[derive(Deserialize)]
pub struct TxResponse {
    id: String,
}

impl<T: Signer> Bundlr<T> {
    pub fn new(url: String, chain: String, currency: String, signer: T) -> Bundlr<T> {
        Bundlr {
            url,
            chain,
            currency,
            signer,
            client: reqwest::Client::new(),
        }
    }

    pub fn create_transaction_with_tags(&self, data: Vec<u8>, tags: Vec<Tag>) -> BundlrTx {
        BundlrTx::create_with_tags(data, tags, &self.signer)
    }

    pub async fn send_transaction(&self, tx: BundlrTx) -> Result<Value, BundlrError> {
        let tx = tx.into_inner();
        let mut f = File::create("test_item").unwrap();
        f.write_all(tx.clone().as_ref()).unwrap();

        let response = self
            .client
            .post(format!("{}/tx/{}", self.url, self.chain))
            .header("Content-Type", "application/octet-stream")
            .body(tx)
            .send()
            .await;

        match response {
            Ok(r) => {
                if !r.status().is_success() {
                    return Err(BundlrError::ResponseError);
                };
                r.json::<Value>()
                    .await
                    .map_err(|_| BundlrError::ResponseError)
            }
            Err(_) => Err(BundlrError::ResponseError),
        }
    }
}
