use std::{pin::Pin, time::Duration};

use bundlr_sdk::{
    client::{balance::run_balance, fund::run_fund, method::Method},
    currency::CurrencyType,
    error::BundlrError,
};
use clap::Parser;
use futures::Future;
use num::BigUint;
use num_traits::Zero;

#[derive(Clone, Debug, Parser)]
#[clap(name = "cli")]
#[clap(about = "", long_about = None)]
struct Args {
    #[clap(value_parser)]
    method: Method,

    #[clap(value_parser)]
    address: Option<String>,

    #[clap(value_parser)]
    amount: Option<BigUint>,

    #[clap(long = "timeout")]
    timeout: Option<u64>,

    #[clap(short = 'w', long = "wallet")]
    wallet: Option<String>,

    #[clap(short = 'h', long = "host")]
    host: String,

    #[clap(short = 'c', long = "currency")]
    currency: CurrencyType,
}

#[tokio::main]
pub async fn main() {
    let args = Args::parse();
    let method = args.method;
    let address = match method {
        Method::Balance => args.address.expect("Argument <Address> not provided"),
        _ => "".to_string(),
    };
    let amount = match method {
        Method::Fund => args.amount.expect("Argument <Amount> not provided"),
        _ => Zero::zero(),
    };
    let wallet = match method {
        Method::Balance => "".to_string(),
        _ => args.wallet.expect("Argument <Wallet> not provided"),
    };
    let url = args.host;
    let timeout = args.timeout.unwrap_or_else(|| 30000);
    let currency = args.currency;

    let (info, work): (
        &str,
        Pin<Box<dyn Future<Output = Result<String, BundlrError>>>>,
    ) = match method {
        Method::Balance => (
            "Balance: ",
            Box::pin(run_balance(&url, &address, &currency)),
        ),
        Method::Fund => (
            "Fund: ",
            Box::pin(run_fund(amount, &url, &wallet, currency)),
        ),
        Method::Withdraw => todo!("Method {:?} not implemented yet", method),
        Method::Help => todo!("Method {:?} not implemented yet", method),
        Method::Upload => todo!("Method {:?} not implemented yet", method),
        Method::UploadDir => todo!("Method {:?} not implemented yet", method),
        Method::Deploy => todo!("Method {:?} not implemented yet", method),
        Method::Price => todo!("Method {:?} not implemented yet", method),
    };

    match tokio::time::timeout(Duration::from_millis(timeout), task).await {
        Ok(res) => println!("{}{:?}", info, res.unwrap()),
        Err(err) => println!("Error: {}", err.to_string()),
    }
}
