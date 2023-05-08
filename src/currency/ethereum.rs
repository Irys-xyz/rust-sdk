use bytes::Bytes;
use reqwest::{StatusCode, Url};

use crate::{
    error::{BuilderError, BundlrError},
    transaction::{Tx, TxStatus},
    Ed25519Signer, Secp256k1Signer, Signer, Verifier,
};

use super::{Currency, CurrencyType, TxResponse};

const ETHEREUM_TICKER: &str = "ETH";
const ETHEREUM_BASE_UNIT: &str = "wei";
const ETHEREUM_BASE_URL: &str = "https://etherscan.io/";

#[allow(unused)]
pub struct Ethereum {
    signer: Option<Secp256k1Signer>,
    is_slow: bool,
    needs_fee: bool,
    base: (String, i64),
    name: CurrencyType,
    ticker: String,
    min_confirm: i16,
    client: reqwest::Client,
    url: Url,
}

impl Default for Ethereum {
    fn default() -> Self {
        let url = Url::parse(ETHEREUM_BASE_URL).unwrap();
        Self {
            signer: None,
            needs_fee: true,
            is_slow: false,
            base: (ETHEREUM_BASE_UNIT.to_string(), 0),
            name: CurrencyType::Ethereum,
            ticker: ETHEREUM_TICKER.to_string(),
            min_confirm: 10,
            client: reqwest::Client::new(),
            url,
        }
    }
}

#[derive(Default)]
pub struct EthereumBuilder {
    base_url: Option<Url>,
    wallet: Option<String>,
}

impl EthereumBuilder {
    pub fn new() -> EthereumBuilder {
        Default::default()
    }

    pub fn base_url(mut self, base_url: Url) -> EthereumBuilder {
        self.base_url = Some(base_url);
        self
    }

    pub fn wallet(mut self, wallet: &str) -> EthereumBuilder {
        self.wallet = Some(wallet.into());
        self
    }

    pub fn build(self) -> Result<Ethereum, BuilderError> {
        let signer = if let Some(wallet) = self.wallet {
            Some(Secp256k1Signer::from_base58(&wallet)?)
        } else {
            None
        };
        Ok(Ethereum {
            url: self
                .base_url
                .unwrap_or_else(|| Url::parse(ETHEREUM_BASE_URL).unwrap()),
            signer,
            ..Ethereum::default()
        })
    }
}

#[allow(unused)]
#[async_trait::async_trait]
impl Currency for Ethereum {
    fn get_min_unit_name(&self) -> String {
        ETHEREUM_BASE_UNIT.to_string()
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

    fn sign_message(&self, message: &[u8]) -> Result<Vec<u8>, BundlrError> {
        match &self.signer {
            Some(signer) => Ok(signer.sign(Bytes::copy_from_slice(message))?.to_vec()),
            None => Err(BundlrError::CurrencyError(
                "No private key present".to_string(),
            )),
        }
    }

    fn verify(&self, pub_key: &[u8], message: &[u8], signature: &[u8]) -> Result<(), BundlrError> {
        Ed25519Signer::verify(
            Bytes::copy_from_slice(pub_key),
            Bytes::copy_from_slice(message),
            Bytes::copy_from_slice(signature),
        )
        .map(|_| ())
    }

    fn get_pub_key(&self) -> Result<Bytes, BundlrError> {
        match &self.signer {
            Some(signer) => Ok(signer.pub_key()),
            None => Err(BundlrError::CurrencyError(
                "No private key present".to_string(),
            )),
        }
    }

    fn wallet_address(&self) -> Result<String, BundlrError> {
        todo!();
    }

    fn get_signer(&self) -> Result<&dyn Signer, BundlrError> {
        match &self.signer {
            Some(signer) => Ok(signer),
            None => Err(BundlrError::CurrencyError(
                "No private key present".to_string(),
            )),
        }
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

    async fn get_fee(&self, _amount: u64, to: &str, multiplier: f64) -> Result<u64, BundlrError> {
        todo!();
    }

    async fn create_tx(&self, amount: u64, to: &str, fee: u64) -> Result<Tx, BundlrError> {
        todo!();
    }

    async fn send_tx(&self, data: Tx) -> Result<TxResponse, BundlrError> {
        todo!()
    }
}
