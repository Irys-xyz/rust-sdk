use arweave_rs::{crypto::base64::Base64, Arweave as ArweaveSdk};
use reqwest::Url;
use std::{path::PathBuf, str::FromStr};

use crate::{error::BundlrError, transaction::Tx, Signer};

use super::{Currency, CurrencyType, TxResponse};

pub struct Arweave {
    sdk: ArweaveSdk,
    is_slow: bool,
    needs_fee: bool,
    base: (String, i64),
    name: CurrencyType,
    ticker: String,
    min_confirm: i16,
    url: Url,
    client: reqwest::Client, //TODO: change this field type to Url
}

impl Default for Arweave {
    fn default() -> Self {
        Self {
            sdk: ArweaveSdk::default(),
            needs_fee: true,
            is_slow: false,
            base: ("winston".to_string(), 0),
            name: CurrencyType::Arweave,
            ticker: "ar".to_string(),
            min_confirm: 5,
            url: Url::from_str("http://arweave.net/v2/transactions").unwrap(),
            client: reqwest::Client::new(),
        }
    }
}

impl Arweave {
    pub fn new(keypair_path: PathBuf, base_url: Url) -> Self {
        Self {
            sdk: ArweaveSdk::from_keypair_path(keypair_path, base_url.clone())
                .expect("Invalid path or url"),
            needs_fee: true,
            is_slow: false,
            base: ("winston".to_string(), 0),
            name: CurrencyType::Arweave,
            ticker: "ar".to_string(),
            min_confirm: 5,
            url: base_url,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl Currency for Arweave {
    fn get_type(&self) -> CurrencyType {
        self.name
    }

    fn needs_fee(&self) -> bool {
        self.needs_fee
    }

    fn get_tx(&self, tx_id: String) -> Tx {
        todo!()
    }

    fn owner_to_address(&self, owner: String) -> String {
        todo!()
    }

    fn get_signer(&self) -> &dyn Signer {
        todo!()
    }

    async fn get_id(&self, item: ()) -> String {
        todo!();
    }

    async fn price(&self) -> String {
        todo!();
    }

    async fn get_current_height(&self) -> u128 {
        todo!();
    }

    async fn get_fee(&self, _amount: u64, _to: &str, multiplier: f64) -> u64 {
        todo!()
    }

    async fn create_tx(&self, amount: u64, to: &str, fee: u64) -> Tx {
        let tx = self
            .sdk
            .create_transaction(
                Base64::from_str(to).unwrap(),
                vec![],
                vec![],
                amount,
                fee,
                false,
            )
            .await
            .expect("Could not create transaction");

        Tx {
            id: tx.id.to_string(),
            from: tx.owner.to_string(),
            to: tx.target.to_string(),
            amount: u64::from_str(&tx.quantity.to_string()).expect("Could not parse amount"),
            block_height: Default::default(),
            pending: true,
            confirmed: false,
        }
    }

    async fn send_tx(&self, data: Tx) -> Result<TxResponse, BundlrError> {
        todo!();
    }
}
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn should_get_fee_correctly() {}
}
