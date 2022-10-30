use std::{pin::Pin, time::Duration};

use bundlr_sdk::{
    client::{
        balance::run_balance, fund::run_fund, method::Method, price::run_price, upload::run_upload,
        withdraw::run_withdraw,
    },
    currency::CurrencyType,
    error::BundlrError,
};
use clap::Parser;
use futures::Future;
use reqwest::Url;

const DEFAULT_TIMEOUT: u64 = 1000 * 30; //30 secs
const DEFAULT_TIMEOUT_FUND: u64 = 1000 * 60 * 30; //30 mins

#[derive(Clone, Debug, Parser)]
#[clap(name = "cli")]
#[clap(about = "", long_about = None)]
struct Args {
    #[clap(value_parser)]
    method: Method,

    #[clap(value_parser)]
    first_arg: Option<String>,

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
        Method::Fund | Method::Withdraw => args.first_arg.expect("Argument <Amount> not provided"),
        Method::Upload => args.first_arg.expect("Argument <File> not provided"),
        _ => "".to_string(),
    };

    let wallet = match method {
        Method::Balance => "".to_string(),
        Method::Price => "".to_string(),
        Method::Help => "".to_string(),
        _ => args.wallet.expect("Argument <Wallet> not provided"),
    };
    let bundlr_url = args.host;
    let timeout = args.timeout.unwrap_or_else(|| match method {
        Method::Fund => DEFAULT_TIMEOUT_FUND,
        _ => DEFAULT_TIMEOUT,
    });
    let currency = args.currency;

    let (info, work): (
        &str,
        Pin<Box<dyn Future<Output = Result<String, BundlrError>>>>,
    ) = match method {
        Method::Balance => (
            "Balance:",
            Box::pin(run_balance(bundlr_url, &first_arg, &currency)),
        ),
        Method::Fund => {
            let amount = u64::from_str_radix(&first_arg, 10).expect("Invalid amount");
            (
                "Fund:",
                Box::pin(run_fund(amount, bundlr_url, &wallet, currency)),
            )
        }
        Method::Withdraw => {
            let amount = u64::from_str_radix(&first_arg, 10).expect("Invalid amount");
            (
                "Withdraw:",
                Box::pin(run_withdraw(amount, bundlr_url, &wallet, currency)),
            )
        }
        Method::Help => todo!("Method {:?} not implemented yet", method),
        Method::Upload => {
            let file = first_arg.to_string();
            (
                "Upload:",
                Box::pin(run_upload(file, bundlr_url, &wallet, currency)),
            )
        }
        Method::UploadDir => todo!("Method {:?} not implemented yet", method),
        Method::Price => {
            let amount = u64::from_str_radix(&first_arg, 10).expect("Invalid amount");
            ("Price:", Box::pin(run_price(bundlr_url, currency, amount)))
        }
    };

    match tokio::time::timeout(Duration::from_millis(timeout), work).await {
        Ok(res) => match res {
            Ok(ok) => println!("[Ok] {} {:?}", info, ok),
            Err(err) => println!("[Err] {} {:?}", info, err),
        },
        Err(err) => println!("Error running task: {} {}", info, err.to_string()),
    }
}
