use std::time::Duration;

use bundlr_sdk::{currency::Currency, Bundlr};
use clap::{ArgEnum, Parser};

#[derive(Clone, Debug, Parser)]
#[clap(name = "balance")]
#[clap(about = "Gets the specified user's balance for the current Bundlr node", long_about = None)]
struct Args {
    #[clap(value_parser)]
    address: String,

    #[clap(short = 'h', long = "host")]
    host: String,

    #[clap(short = 'c', long = "currency")]
    currency: String,

    #[clap(long = "timeout")]
    timeout: Option<u64>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let url = args.host;
    let address = args.address;
    let currency = Currency::from_str(&args.currency, false).unwrap();
    let timeout = args.timeout.unwrap_or_else(|| 30000);

    let bundler = &Bundlr::new(url, currency, None).await;
    let work = bundler.get_balance(address);

    match tokio::time::timeout(Duration::from_millis(timeout), work).await {
        Ok(result) => {
            println!("Balance: {:?}", result.unwrap());
        }
        Err(err) => {
            println!("Error getting balance: {:?}", err.to_string());
        }
    }
}
