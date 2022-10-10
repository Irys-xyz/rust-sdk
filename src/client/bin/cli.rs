use std::{pin::Pin, time::Duration};

use bundlr_sdk::{
    client::{balance::run_balance, fund::run_fund, method::Method, price::run_price},
    currency::CurrencyType,
    error::BundlrError,
};
use clap::Parser;
use futures::Future;
use reqwest::Url;

const DEFAULT_TIMEOUT: u64 = 60000 * 20; //20 mins

#[derive(Clone, Debug, Parser)]
#[clap(name = "cli")]
#[clap(about = "", long_about = None)]
struct Args {
    #[clap(value_parser)]
    method: Method,

    #[clap(value_parser)]
    first_arg: Option<String>,

    #[clap(value_parser)]
    second_arg: Option<String>,

    #[clap(long = "timeout")]
    timeout: Option<u64>,

    #[clap(short = 'w', long = "wallet")]
    wallet: Option<String>,

    #[clap(short = 'h', long = "host")]
    host: Url,

    #[clap(short = 'c', long = "currency")]
    currency: CurrencyType,
}

#[tokio::main]
pub async fn main() {
    let args = Args::parse();
    let method = args.method;
    let first_arg = match method {
        Method::Balance => args.first_arg.expect("Argument <Address> not provided"),
        Method::Price => args.first_arg.expect("Argument <Amount> not provided"),
        _ => "".to_string(),
    };
    let second_arg = match method {
        Method::Fund => args.second_arg.expect("Argument <Amount> not provided"),
        _ => "0".to_string(),
    };
    let wallet = match method {
        Method::Balance => "".to_string(),
        Method::Price => "".to_string(),
        _ => args.wallet.expect("Argument <Wallet> not provided"),
    };
    let bundlr_url = args.host;
    let timeout = args.timeout.unwrap_or(DEFAULT_TIMEOUT);
    let currency = args.currency;

    let (info, work): (
        &str,
        Pin<Box<dyn Future<Output = Result<String, BundlrError>>>>,
    ) = match method {
        Method::Balance => (
            "Balance: ",
            Box::pin(run_balance(bundlr_url, &first_arg, &currency)),
        ),
        Method::Fund => {
            let amount = u64::from_str_radix(&first_arg, 10).expect("Invalid amount");
            (
                "Fund: ",
                Box::pin(run_fund(amount, bundlr_url, &wallet, currency)),
            )
        }
        Method::Withdraw => todo!("Method {:?} not implemented yet", method),
        Method::Help => todo!("Method {:?} not implemented yet", method),
        Method::Upload => todo!("Method {:?} not implemented yet", method),
        Method::UploadDir => todo!("Method {:?} not implemented yet", method),
        Method::Deploy => todo!("Method {:?} not implemented yet", method),
        Method::Price => {
            let amount = u64::from_str_radix(&first_arg, 10).expect("Invalid amount");
            ("Price: ", Box::pin(run_price(bundlr_url, currency, amount)))
        }
    };

    match tokio::time::timeout(Duration::from_millis(timeout), work).await {
        Ok(res) => println!("{}{:?}", info, res.unwrap()),
        Err(err) => println!("Error: {}", err.to_string()),
    }
}
