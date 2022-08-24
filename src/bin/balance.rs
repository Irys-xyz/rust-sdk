use std::str::FromStr;

use bundlr_sdk::{currency::Currency, Bundlr};
use clap::Parser;

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
    timeout: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let url = args.host;
    let address = args.address;
    let currency = Currency::from_str(&args.currency).unwrap();

    let bundler = &Bundlr::new(url, currency, None);
    let balance = bundler.get_balance(address).await.unwrap();

    println!("Balance: {:?}", balance);
}
