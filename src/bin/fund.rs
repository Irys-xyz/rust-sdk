use std::{str::FromStr, time::Duration};

use bundlr_sdk::{
    currency::Currency, wallet::load_from_file, ArweaveSigner, Bundlr, CosmosSigner, Ed25519Signer,
    Secp256k1Signer, Signer,
};
use clap::{ArgEnum, Parser};
use num_bigint::BigUint;
use num_traits::Zero;

#[derive(Clone, Debug, Parser)]
#[clap(name = "fund")]
#[clap(about = "Funds your account with the specified amount of atomic units", long_about = None)]
struct Args {
    #[clap(value_parser)]
    amount: String,

    #[clap(short = 'h', long = "host")]
    host: String,

    #[clap(short = 'w', long = "wallet")]
    wallet: String,

    #[clap(short = 'c', long = "currency")]
    currency: String,

    #[clap(long = "timeout")]
    timeout: Option<u64>,
}

fn confirm(amount: &BigUint, currency: &Currency, host: &String, address: &String) -> bool {
    let mut line = String::new();
    println!(
        "Confirmation: send {} {} to {} {}?",
        amount, currency, host, address
    );
    println!("Y/N: ");
    let mut bl = std::io::stdin().read_line(&mut line).unwrap();
    while bl < 2 || bl > 4 {
        println!("Y/N: ");
        bl = std::io::stdin().read_line(&mut line).unwrap();
    }
    match line.trim().to_string().as_str() {
        "y" | "Y" | "yes" | "Yes" => true,
        _ => false,
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let amount = args.amount.parse::<BigUint>().unwrap();
    let currency = Currency::from_str(&args.currency, false).unwrap();
    let url = args.host;
    let wallet = args.wallet;
    let timeout = args.timeout.unwrap_or_else(|| 30000);

    if amount.le(&Zero::zero()) {
        println!("Funding amount should be valid");
        return;
    }

    let confirmed = confirm(&amount, &currency, &url, &String::from("address"));
    if !confirmed {
        println!("Confirmation failed");
        return;
    }

    let signer: Option<Box<dyn Signer>> = match currency {
        Currency::Arweave => {
            let wallet_path = wallet;
            let jwk = load_from_file(&wallet_path);
            let signer = Box::new(ArweaveSigner::from_jwk(jwk));
            Some(signer)
        }
        Currency::Solana => Some(Box::new(Ed25519Signer::from_base58(&wallet))),
        Currency::Ethereum | Currency::Erc20 => {
            Some(Box::new(Secp256k1Signer::from_base58(&wallet)))
        }
        Currency::Cosmos => Some(Box::new(CosmosSigner::from_base58(&wallet))),
    };
    let currency = Currency::from(currency);
    let bundler = &Bundlr::new(url, currency, signer).await;
    let work = bundler.fund(amount);

    match tokio::time::timeout(Duration::from_millis(timeout), work).await {
        Ok(result) => {
            println!("Balance: {:?}", result.unwrap());
        }
        Err(err) => {
            println!("Error getting balance: {:?}", err.to_string());
        }
    }
}
