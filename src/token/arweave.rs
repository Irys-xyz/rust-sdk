use arweave_rs::{crypto::base64::Base64, Arweave as ArweaveSdk};
use bytes::Bytes;
use num::ToPrimitive;
use reqwest::{StatusCode, Url};
use std::{ops::Mul, path::PathBuf, str::FromStr};

use crate::{
    error::{BuilderError, BundlerError},
    transaction::{Tx, TxStatus},
    ArweaveSigner, Signer, Verifier,
};

use super::{Token, TokenType, TxResponse};

const ARWEAVE_TICKER: &str = "AR";
const ARWEAVE_BASE_UNIT: &str = "winston";
const ARWEAVE_BASE_URL: &str = "https://arweave.net/";

#[allow(unused)]
pub struct Arweave {
    sdk: ArweaveSdk,
    signer: Option<ArweaveSigner>,
    is_slow: bool,
    needs_fee: bool,
    base: (String, i64),
    name: TokenType,
    ticker: String,
    min_confirm: i16,
    client: reqwest::Client,
}

#[derive(Default)]
pub struct ArweaveBuilder {
    base_url: Option<Url>,
    keypair_path: Option<PathBuf>,
}

impl ArweaveBuilder {
    pub fn new() -> ArweaveBuilder {
        Default::default()
    }

    pub fn base_url(mut self, base_url: Url) -> ArweaveBuilder {
        self.base_url = Some(base_url);
        self
    }

    pub fn keypair_path(mut self, keypair_path: PathBuf) -> ArweaveBuilder {
        self.keypair_path = Some(keypair_path);
        self
    }

    pub fn build(self) -> Result<Arweave, BuilderError> {
        let base_url = self
            .base_url
            .unwrap_or_else(|| Url::from_str(ARWEAVE_BASE_URL).unwrap());

        let sdk = match &self.keypair_path {
            // With signer
            Some(keypair_path) => arweave_rs::ArweaveBuilder::new()
                .base_url(base_url)
                .keypair_path(keypair_path.clone())
                .build()?,
            // Without signer
            None => arweave_rs::ArweaveBuilder::new()
                .base_url(base_url)
                .build()?,
        };

        let signer = match self.keypair_path {
            Some(p) => Some(ArweaveSigner::from_keypair_path(p)?),
            None => None,
        };

        Ok(Arweave {
            sdk,
            signer,
            is_slow: Default::default(),
            needs_fee: true,
            base: (ARWEAVE_BASE_UNIT.to_string(), 0),
            name: TokenType::Arweave,
            ticker: ARWEAVE_TICKER.to_string(),
            min_confirm: 5,
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait::async_trait]
impl Token for Arweave {
    fn get_min_unit_name(&self) -> String {
        ARWEAVE_BASE_UNIT.to_string()
    }

    fn get_type(&self) -> TokenType {
        self.name
    }

    fn needs_fee(&self) -> bool {
        self.needs_fee
    }

    async fn get_tx(&self, tx_id: String) -> Result<Tx, BundlerError> {
        let (status, tx) = self
            .sdk
            .get_tx(
                Base64::from_str(&tx_id)
                    .map_err(|err| BundlerError::ParseError(err.to_string()))?,
            )
            .await
            .map_err(BundlerError::ArweaveSdkError)?;

        if status == 200 {
            match tx {
                Some(tx) => Ok(Tx {
                    id: tx.id.to_string(),
                    from: tx.owner.to_string(),
                    to: tx.target.to_string(),
                    amount: u64::from_str(&tx.quantity.to_string())
                        .map_err(|err| BundlerError::ParseError(err.to_string()))?,
                    fee: tx.reward,
                    block_height: 1,
                    pending: false,
                    confirmed: true,
                }),
                None => Err(BundlerError::TxNotFound),
            }
        } else {
            Err(BundlerError::TxNotFound)
        }
    }

    async fn get_tx_status(
        &self,
        tx_id: String,
    ) -> Result<(StatusCode, Option<TxStatus>), BundlerError> {
        let res = self
            .sdk
            .get_tx_status(
                Base64::from_str(&tx_id)
                    .map_err(|err| BundlerError::ParseError(err.to_string()))?,
            )
            .await;

        if let Ok((status, tx_status)) = res {
            if status == StatusCode::OK {
                match tx_status {
                    Some(tx_status) => Ok((
                        status,
                        Some(TxStatus {
                            confirmations: tx_status.number_of_confirmations,
                            height: tx_status.block_height,
                            block_hash: tx_status.block_indep_hash.to_string(),
                        }),
                    )),
                    None => Ok((status, None)),
                }
            } else {
                //Tx is pending
                Ok((status, None))
            }
        } else {
            Err(BundlerError::TxStatusNotConfirmed)
        }
    }

    fn sign_message(&self, message: &[u8]) -> Result<Vec<u8>, BundlerError> {
        match &self.signer {
            Some(signer) => Ok(signer.sign(Bytes::copy_from_slice(message))?.to_vec()),
            None => Err(BundlerError::TokenError(
                "No private key present".to_string(),
            )),
        }
    }

    fn verify(&self, pub_key: &[u8], message: &[u8], signature: &[u8]) -> Result<(), BundlerError> {
        ArweaveSigner::verify(
            Bytes::copy_from_slice(pub_key),
            Bytes::copy_from_slice(message),
            Bytes::copy_from_slice(signature),
        )
        .map(|_| ())
    }

    fn get_pub_key(&self) -> Result<Bytes, BundlerError> {
        match &self.signer {
            Some(signer) => Ok(signer.pub_key()),
            None => Err(BundlerError::TokenError(
                "No private key present".to_string(),
            )),
        }
    }

    fn wallet_address(&self) -> Result<String, BundlerError> {
        if self.signer.is_none() {
            return Err(BundlerError::TokenError(
                "No private key present".to_string(),
            ));
        }
        Ok(self.sdk.get_wallet_address()?)
    }

    fn get_signer(&self) -> Result<&dyn Signer, BundlerError> {
        match &self.signer {
            Some(signer) => Ok(signer),
            None => Err(BundlerError::TokenError(
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

    async fn get_fee(&self, _amount: u64, to: &str, multiplier: f64) -> Result<u64, BundlerError> {
        let base64_address =
            Base64::from_str(to).map_err(|err| BundlerError::ParseError(err.to_string()))?;
        let base_fee = self
            .sdk
            .get_fee(base64_address, vec![])
            .await
            .map_err(BundlerError::ArweaveSdkError)?;

        let fee = match base_fee.to_f64() {
            Some(ok) => ok,
            None => {
                return Err(BundlerError::TypeParseError(
                    "Could not convert to f64".to_string(),
                ))
            }
        };
        let final_fee = match multiplier.mul(fee).ceil().to_u64() {
            Some(fee) => fee,
            None => {
                return Err(BundlerError::TypeParseError(
                    "Could not convert fee to u64".to_string(),
                ))
            }
        };
        Ok(final_fee)
    }

    async fn create_tx(&self, amount: u64, to: &str, fee: u64) -> Result<Tx, BundlerError> {
        let tx = self
            .sdk
            .create_transaction(
                Base64::from_str(to).map_err(|err| BundlerError::Base64Error(err.to_string()))?,
                vec![],
                vec![],
                amount.into(),
                fee,
                false,
            )
            .await
            .map_err(BundlerError::ArweaveSdkError)?;

        Ok(Tx {
            id: tx.id.to_string(),
            from: tx.owner.to_string(),
            to: tx.target.to_string(),
            amount: u64::from_str(&tx.quantity.to_string())
                .map_err(|err| BundlerError::Base64Error(err.to_string()))?,
            fee: tx.reward,
            block_height: Default::default(),
            pending: true,
            confirmed: false,
        })
    }

    async fn send_tx(&self, data: Tx) -> Result<TxResponse, BundlerError> {
        let tx = self
            .sdk
            .create_transaction(
                Base64::from_str(&data.to)
                    .map_err(|err| BundlerError::Base64Error(err.to_string()))?,
                vec![],
                vec![],
                data.amount.into(),
                data.fee,
                false,
            )
            .await
            .map_err(BundlerError::ArweaveSdkError)?;

        let signed_tx = self
            .sdk
            .sign_transaction(tx)
            .map_err(BundlerError::ArweaveSdkError)?;
        let (tx_id, _r) = self
            .sdk
            .post_transaction(&signed_tx)
            .await
            .map_err(BundlerError::ArweaveSdkError)?;

        Ok(TxResponse { tx_id })
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use crate::token::{arweave::ArweaveBuilder, Token};

    #[test]
    fn should_sign_and_verify() {
        let msg = [
            9, 214, 233, 210, 242, 45, 194, 247, 28, 234, 14, 86, 105, 40, 41, 251, 52, 39, 236,
            214, 54, 13, 53, 254, 179, 53, 220, 205, 129, 37, 244, 142, 230, 32, 209, 103, 68, 75,
            39, 178, 10, 186, 24, 160, 179, 143, 211, 151,
        ];
        let wallet = PathBuf::from_str("res/test_wallet.json").expect("Could not load path");
        let c = ArweaveBuilder::new()
            .keypair_path(wallet)
            .build()
            .expect("Could not build arweave");

        let sig = c.sign_message(&msg).unwrap();
        let pub_key = c.get_pub_key().unwrap();

        assert!(c.verify(&pub_key, &msg, &sig).is_ok());
    }

    #[tokio::test]
    async fn should_get_fee_correctly() {}
}
