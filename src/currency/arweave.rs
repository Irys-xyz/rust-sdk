use std::str::FromStr;

use crate::{error::BundlrError, transaction::Tx, ArweaveSigner, Signer};
use num::{BigRational, BigUint, CheckedMul, One};

use super::{Currency, CurrencyType};

pub struct Arweave<'a> {
    is_slow: bool,
    needs_fee: bool,
    base: (String, i64),
    name: CurrencyType,
    ticker: String,
    signer: Option<&'a ArweaveSigner>,
}

impl<'a> Arweave<'a> {
    pub fn new(s: Option<&'a ArweaveSigner>) -> Self {
        Self {
            needs_fee: true,
            is_slow: false,
            base: ("winston".to_string(), 0),
            name: CurrencyType::Arweave,
            ticker: "ar".to_string(),
            signer: s,
        }
    }
}

#[async_trait::async_trait]
impl<'a> Currency for Arweave<'a> {
    fn get_type(&self) -> CurrencyType {
        self.name
    }
    fn needs_fee(&self) -> bool {
        todo!();
    }
    fn get_tx(&self, tx_id: String) -> Tx {
        todo!();
    }
    fn owner_to_address(&self, owner: String) -> String {
        todo!()
    }
    fn get_signer(&self) -> &'a dyn Signer {
        self.signer.expect("No signer present")
    }
    async fn get_id(&self, item: ()) -> String {
        todo!();
    }
    async fn price(&self) -> String {
        todo!();
    }
    async fn get_current_height(&self) -> BigUint {
        todo!();
    }
    async fn get_fee(
        &self,
        _amount: &BigUint,
        _to: &str,
        multiplier: Option<BigRational>,
    ) -> BigUint {
        let base_fee: BigUint = One::one(); //TODO: get fee properly
        if multiplier.is_some() {
            let multiplier = multiplier.unwrap();
            let base_fee = BigRational::from_str(&base_fee.to_string())
                .expect("Error converting BigUInt to BigFloat");
            let base_fee = base_fee
                .checked_mul(&multiplier)
                .expect("Error multiplying two BigRational numbers");
            let base_fee = base_fee.ceil();
            BigUint::from_str(&base_fee.to_string()).expect("Error converting BigInt to BigUint")
        } else {
            base_fee.clone()
        }
    }
    async fn create_tx(&self, _amount: &BigUint, _to: &str, _fee: &BigUint) -> Tx {
        todo!();
    }
    async fn send_tx(&self, data: Vec<u8>) -> Result<bool, BundlrError> {
        todo!();
    }
}
