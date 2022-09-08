use crate::{
    currency::{arweave::Arweave, Currency, CurrencyType},
    error::BundlrError,
    wallet::load_from_file,
    ArweaveSigner, Bundlr,
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

    let jwk = load_from_file(wallet).unwrap();
    let signer = ArweaveSigner::from_jwk(jwk);
    let currency: Box<dyn Currency> = match currency {
        CurrencyType::Arweave => Box::new(Arweave::new(Some(&signer))),
        CurrencyType::Solana => todo!(),
        CurrencyType::Ethereum => todo!(),
        CurrencyType::Erc20 => todo!(),
        CurrencyType::Cosmos => todo!(),
    };
    let bundlr = Bundlr::new(url.to_string(), currency.as_ref()).await;

    bundlr.fund(amount, None).await.map(|res| res.to_string())
}
