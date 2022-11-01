use bytes::Bytes;
use reqwest::{StatusCode, Url};

use crate::{
    error::BundlrError,
    transaction::{Tx, TxStatus},
    Ed25519Signer, Signer, Verifier,
};

use super::{Currency, CurrencyType, TxResponse};

const SOLANA_TICKER: &str = "SOL";
const SOLANA_BASE_UNIT: &str = "lamport";
const SOLANA_BASE_URL: &str = "https://explorer.solana.com/";

#[allow(unused)]
pub struct Solana {
    signer: Option<Ed25519Signer>,
    is_slow: bool,
    needs_fee: bool,
    base: (String, i64),
    name: CurrencyType,
    ticker: String,
    min_confirm: i16,
    client: reqwest::Client,
    url: Url,
}

impl Default for Solana {
    fn default() -> Self {
        let url = Url::parse(SOLANA_BASE_URL).unwrap();
        Self {
            signer: None,
            needs_fee: true,
            is_slow: false,
            base: (SOLANA_BASE_UNIT.to_string(), 0),
            name: CurrencyType::Arweave,
            ticker: SOLANA_TICKER.to_string(),
            min_confirm: 10,
            client: reqwest::Client::new(),
            url,
        }
    }
}

impl Solana {
    pub fn new(wallet: &str, url: Option<Url>) -> Self {
        let signer = Ed25519Signer::from_base58(&wallet);
        Self {
            signer: Some(signer),
            url: url.unwrap_or(Url::parse(SOLANA_BASE_URL).expect("Could not parse Url")),
            ..Self::default()
        }
    }
}

#[allow(unused)]
#[async_trait::async_trait]
impl Currency for Solana {
    fn get_min_unit_name(&self) -> String {
        SOLANA_BASE_UNIT.to_string()
    }

    fn get_type(&self) -> CurrencyType {
        self.name
    }

    fn needs_fee(&self) -> bool {
        self.needs_fee
    }

    async fn get_tx(&self, tx_id: String) -> Result<Tx, BundlrError> {
        todo!()
    }

    async fn get_tx_status(
        &self,
        tx_id: String,
    ) -> Result<(StatusCode, Option<TxStatus>), BundlrError> {
        todo!()
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
        Ed25519Signer::verify(
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
        todo!();
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
        todo!();
    }

    async fn create_tx(&self, amount: u64, to: &str, fee: u64) -> Tx {
        todo!();
    }

    async fn send_tx(&self, data: Tx) -> Result<TxResponse, BundlrError> {
        todo!()
    }
}
