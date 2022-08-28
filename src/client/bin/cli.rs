use std::time::Duration;

use bundlr_sdk::client::{balance::run_balance, method::Method};
use clap::Parser;

#[derive(Clone, Debug, Parser)]
#[clap(name = "cli")]
#[clap(about = "", long_about = None)]
struct Args {
    #[clap(value_parser)]
    method: Method,

    #[clap(value_parser)]
    address: Option<String>,

    #[clap(long = "timeout")]
    timeout: Option<u64>,

    #[clap(short = 'h', long = "host")]
    host: String,

    #[clap(short = 'c', long = "currency")]
    currency: String,
}

#[tokio::main]
pub async fn main() {
    let args = Args::parse();
    let method = args.method;
    let address = match method {
        Method::Balance => args.address.expect("Argument <Address> not provided"),
        _ => "".to_string(),
    };
    let url = args.host;
    let timeout = args.timeout.unwrap_or_else(|| 30000);
    let currency = args.currency;

    let (info, work) = match method {
        Method::Balance => ("Balance: ", run_balance(&url, &address, &currency)),
        _ => panic!("Method {:?} not recognized or not implemented yet", method),
    };

    match tokio::time::timeout(Duration::from_millis(timeout), work).await {
        Ok(res) => println!("{}{:?}", info, res.unwrap()),
        Err(err) => println!("Error: {}", err.to_string()),
    }
}
