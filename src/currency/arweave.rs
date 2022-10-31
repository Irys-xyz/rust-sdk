use arweave_rs::{crypto::base64::Base64, Arweave as ArweaveSdk};
use bytes::Bytes;
use num::ToPrimitive;
use reqwest::{StatusCode, Url};
use std::{ops::Mul, path::PathBuf, str::FromStr};

use crate::{
    error::BundlrError,
    transaction::{Tx, TxStatus},
    ArweaveSigner, Signer, Verifier,
};

use super::{Currency, CurrencyType, TxResponse};

const ARWEAVE_BASE_UNIT: &str = "winston";
const ARWEAVE_BASE_URL: &str = "https://arweave.net/";

pub struct Arweave {
    sdk: ArweaveSdk,
    signer: Option<ArweaveSigner>,
    is_slow: bool,
    needs_fee: bool,
    base: (String, i64),
    name: CurrencyType,
    ticker: String,
    min_confirm: i16,
    client: reqwest::Client,
}

impl Default for Arweave {
    fn default() -> Self {
        Self {
            sdk: ArweaveSdk::default(),
            signer: None,
            needs_fee: true,
            is_slow: false,
            base: ("winston".to_string(), 0),
            name: CurrencyType::Arweave,
            ticker: "ar".to_string(),
            min_confirm: 5,
            client: reqwest::Client::new(),
        }
    }
}

impl Arweave {
    pub fn new(keypair_path: PathBuf, base_url: Option<Url>) -> Self {
        let base_url = base_url.unwrap_or(Url::from_str(ARWEAVE_BASE_URL).unwrap());
        Self {
            sdk: ArweaveSdk::from_keypair_path(keypair_path.clone(), base_url)
                .expect("Invalid path or url"),
            signer: Some(ArweaveSigner::from_keypair_path(keypair_path).expect("Invalid path")),
            needs_fee: true,
            is_slow: false,
            base: ("winston".to_string(), 0),
            name: CurrencyType::Arweave,
            ticker: "ar".to_string(),
            min_confirm: 5,
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait::async_trait]
impl Currency for Arweave {
    fn get_min_unit_name(&self) -> String {
        ARWEAVE_BASE_UNIT.to_string()
    }

    fn get_type(&self) -> CurrencyType {
        self.name
    }

    fn needs_fee(&self) -> bool {
        self.needs_fee
    }

    async fn get_tx(&self, tx_id: String) -> Result<Tx, BundlrError> {
        let (status, tx) = self
            .sdk
            .get_tx(Base64::from_str(&tx_id).expect("Could not parse tx_id into base64"))
            .await
            .expect("Could not get tx");

        if status == 200 {
            let tx = tx.unwrap();
            Ok(Tx {
                id: tx.id.to_string(),
                from: tx.owner.to_string(),
                to: tx.target.to_string(),
                amount: u64::from_str(&tx.quantity.to_string()).expect("Could not parse amount"),
                fee: tx.reward,
                block_height: 1,
                pending: false,
                confirmed: true,
            })
        } else {
            Err(BundlrError::TxNotFound)
        }
    }

    async fn get_tx_status(
        &self,
        tx_id: String,
    ) -> Result<(StatusCode, Option<TxStatus>), BundlrError> {
        let res = self
            .sdk
            .get_tx_status(Base64::from_str(&tx_id).expect("Could not parse tx_id into base64"))
            .await;

        if let Ok((status, tx_status)) = res {
            if status == StatusCode::OK {
                let tx_status = tx_status.unwrap();
                Ok((
                    status,
                    Some(TxStatus {
                        confirmations: tx_status.number_of_confirmations,
                        height: tx_status.block_height,
                        block_hash: tx_status.block_indep_hash.to_string(),
                    }),
                ))
            } else {
                //Tx is pending
                Ok((status, None))
            }
        } else {
            Err(BundlrError::TxStatusNotConfirmed)
        }
    }

    fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        self.signer
            .as_ref()
            .expect("No signer present")
            .sign(Bytes::copy_from_slice(message))
            .expect("Could not sign message")
            .to_vec()
    }

    fn verify(&self, pub_key: &[u8], message: &[u8], signature: &[u8]) -> Result<(), BundlrError> {
        ArweaveSigner::verify(
            Bytes::copy_from_slice(pub_key),
            Bytes::copy_from_slice(message),
            Bytes::copy_from_slice(signature),
        )
        .map(|_| ())
    }

    fn get_pub_key(&self) -> Bytes {
        self.signer.as_ref().expect("No signer present").pub_key()
    }

    fn wallet_address(&self) -> String {
        self.sdk.get_wallet_address()
    }

    fn get_signer(&self) -> &dyn Signer {
        self.signer.as_ref().expect("No signer present")
    }

    async fn get_id(&self, _item: ()) -> String {
        todo!();
    }

    async fn price(&self) -> String {
        todo!();
    }

    async fn get_current_height(&self) -> u128 {
        todo!();
    }

    async fn get_fee(&self, _amount: u64, to: &str, multiplier: f64) -> u64 {
        let base64_address = Base64::from_str(to).expect("Could not convert target to base64");
        let base_fee = self
            .sdk
            .get_fee(base64_address, vec![])
            .await
            .expect("Could not get fee");
        multiplier
            .mul(base_fee.to_f64().unwrap())
            .ceil()
            .to_u64()
            .unwrap()
    }

    async fn create_tx(&self, amount: u64, to: &str, fee: u64) -> Tx {
        let tx = self
            .sdk
            .create_transaction(
                Base64::from_str(to).unwrap(),
                vec![],
                vec![],
                amount.into(),
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
            fee: tx.reward,
            block_height: Default::default(),
            pending: true,
            confirmed: false,
        }
    }

    async fn send_tx(&self, data: Tx) -> Result<TxResponse, BundlrError> {
        let tx = self
            .sdk
            .create_transaction(
                Base64::from_str(&data.to).expect("Could not convert to Base64"),
                vec![],
                vec![],
                data.amount.into(),
                data.fee,
                false,
            )
            .await
            .expect("Could not create transaction");

        let signed_tx = self
            .sdk
            .sign_transaction(tx)
            .expect("Could not sign transaction");
        let (tx_id, _r) = self
            .sdk
            .post_transaction(&signed_tx)
            .await
            .expect("Could not send transaction");

        Ok(TxResponse {
            tx_id: tx_id.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use crate::currency::{arweave::Arweave, Currency};

    #[test]
    fn should_sign_and_verify() {
        let msg = [
            9, 214, 233, 210, 242, 45, 194, 247, 28, 234, 14, 86, 105, 40, 41, 251, 52, 39, 236,
            214, 54, 13, 53, 254, 179, 53, 220, 205, 129, 37, 244, 142, 230, 32, 209, 103, 68, 75,
            39, 178, 10, 186, 24, 160, 179, 143, 211, 151,
        ];
        let path = PathBuf::from_str("res/test_wallet.json").expect("Could not load path");
        let c = Arweave::new(path, None);

        let sig = c.sign_message(&msg);
        let pub_key = c.get_pub_key();

        assert!(c.verify(&pub_key, &msg, &sig).is_ok());
    }

    #[tokio::test]
    async fn should_get_fee_correctly() {}
}
