use crate::{
    currency::Currency, error::BundlrError, wallet::load_from_file, ArweaveSigner, Bundlr,
};
use clap::ArgEnum;
use num::BigUint;
use num_traits::Zero;

pub async fn run_fund(
    amount: BigUint,
    url: &str,
    wallet: &str,
    currency: &str,
) -> Result<String, BundlrError> {
    if amount.le(&Zero::zero()) {
        return Err(BundlrError::InvalidAmount);
    }

    let currency = Currency::from_str(currency, false).unwrap();
    let jwk = load_from_file(wallet);
    let signer = ArweaveSigner::from_jwk(jwk);
    let bundlr = Bundlr::new(url.to_string(), currency, &signer).await;

    bundlr.fund(amount, None).await.map(|res| res.to_string())
}
