use crate::{
    currency::{arweave::Arweave, Currency, CurrencyType},
    error::BundlrError,
    wallet::load_from_file,
    ArweaveSigner, Bundlr, Signer,
};
use num::BigUint;
use num_traits::Zero;

pub async fn run_fund(
    amount: BigUint,
    url: &str,
    wallet: &str,
    currency: CurrencyType,
) -> Result<String, BundlrError> {
    if amount.le(&Zero::zero()) {
        return Err(BundlrError::InvalidAmount);
    }

    let jwk = load_from_file(wallet);
    let signer = ArweaveSigner::from_jwk(jwk);
    let currency = Arweave::new(Some(&signer));
    let bundlr = Bundlr::new(url.to_string(), &currency).await;

    bundlr.fund(amount, None).await.map(|res| res.to_string())
}
