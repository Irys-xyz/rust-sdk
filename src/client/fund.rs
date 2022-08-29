use crate::{
    currency::Currency, error::BundlrError, wallet::load_from_file, ArweaveSigner, Bundlr,
};
use clap::ArgEnum;
use num_bigint::BigUint;

pub async fn run_fund(
    amount: BigUint,
    url: &str,
    wallet: &str,
    currency: &str,
) -> Result<String, BundlrError> {
    let currency = Currency::from_str(currency, false).unwrap();
    let jwk = load_from_file(wallet);
    let signer = ArweaveSigner::from_jwk(jwk);
    let bundlr = Bundlr::new(url.to_string(), currency, &signer).await;

    bundlr.fund(amount).await.map(|res| res.to_string())
}
