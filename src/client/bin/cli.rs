use std::time::Duration;

use clap::{Parser, Subcommand};
use irys_sdk::{
    client::{
        balance::run_balance, fund::run_fund, price::run_price, upload::run_upload,
        withdraw::run_withdraw,
    },
    token::TokenType,
};
use reqwest::Url;

const DEFAULT_BYTE_AMOUNT: u64 = 1;
const DEFAULT_TIMEOUT: u64 = 1000 * 30; //30 secs
const DEFAULT_TIMEOUT_FUND: u64 = 1000 * 60 * 30; //30 mins

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    ///Gets the specified user's balance for the current Irys bundler node
    Balance {
        //Address to query balance
        #[clap(value_parser)]
        address: String,

        //Timeout for operation
        #[clap(long = "timeout")]
        timeout: Option<u64>,

        //Host address
        #[clap(long = "host")]
        host: Url,

        //Token type
        #[clap(short = 't', long = "token")]
        token: TokenType,
    },
    ///Funds your account with the specified amount of atomic units
    Fund {
        //Amounts, in winston, to send in funding
        #[clap(value_parser)]
        amount: u64,

        //Timeout for operation
        #[clap(long = "timeout")]
        timeout: Option<u64>,

        //Path to wallet
        #[clap(short = 'w', long = "wallet")]
        wallet: String,

        //Host address
        #[clap(long = "host")]
        host: Url,

        //Token type
        #[clap(short = 't', long = "token")]
        token: TokenType,
    },
    ///Sends a fund withdrawal request
    Withdraw {
        //Amounts, in winston, to send in withdraw
        #[clap(value_parser)]
        amount: u64,

        //Timeout for operation
        #[clap(long = "timeout")]
        timeout: Option<u64>,

        //Path to wallet
        #[clap(short = 'w', long = "wallet")]
        wallet: String,

        //Host address
        #[clap(long = "host")]
        host: Url,

        //Token type
        #[clap(short = 't', long = "token")]
        token: TokenType,
    },
    ///Uploads a specified file
    Upload {
        //Path to file that will be uploaded
        #[clap(value_parser)]
        file_path: String,

        //Timeout for operation
        #[clap(long = "timeout")]
        timeout: Option<u64>,

        //Path to wallet
        #[clap(short = 'w', long = "wallet")]
        wallet: String,

        //Host address
        #[clap(long = "host")]
        host: Url,

        //Token type
        #[clap(short = 't', long = "token")]
        token: TokenType,
    },
    ///Uploads a folder (with a manifest)
    UploadDir {},
    ///Check how much of a specific token is required for an upload of <amount> bytes
    Price {
        //Amounts of bytes to calculate pricing
        #[clap(value_parser)]
        byte_amount: Option<u64>,

        //Timeout for operation
        #[clap(long = "timeout")]
        timeout: Option<u64>,

        //Host address
        #[clap(long = "host")]
        host: Url,

        //Token type
        #[clap(short = 't', long = "token")]
        token: TokenType,
    },
}

impl Command {
    async fn execute(self) {
        match self {
            Command::Balance {
                address,
                timeout,
                host,
                token,
            } => {
                let work = run_balance(host, &address, token);
                let timeout = timeout.unwrap_or(DEFAULT_TIMEOUT);
                match tokio::time::timeout(Duration::from_millis(timeout), work).await {
                    Ok(res) => match res {
                        Ok(ok) => println!("[Ok] {}", ok),
                        Err(err) => println!("[Err] {}", err),
                    },
                    Err(err) => println!("Error running task: {}", err),
                }
            }
            Command::Fund {
                amount,
                timeout,
                wallet,
                host,
                token,
            } => {
                let work = run_fund(amount, host, &wallet, token);
                let timeout = timeout.unwrap_or(DEFAULT_TIMEOUT_FUND);
                match tokio::time::timeout(Duration::from_millis(timeout), work).await {
                    Ok(res) => match res {
                        Ok(ok) => println!("[Ok] {}", ok),
                        Err(err) => println!("[Err] {}", err),
                    },
                    Err(err) => println!("Error running task: {}", err),
                }
            }
            Command::Withdraw {
                amount,
                timeout,
                wallet,
                host,
                token,
            } => {
                let work = run_withdraw(amount, host, &wallet, token);
                let timeout = timeout.unwrap_or(DEFAULT_TIMEOUT);
                match tokio::time::timeout(Duration::from_millis(timeout), work).await {
                    Ok(res) => match res {
                        Ok(ok) => println!("[Ok] {}", ok),
                        Err(err) => println!("[Err] {}", err),
                    },
                    Err(err) => println!("Error running task: {}", err),
                }
            }
            Command::Upload {
                file_path,
                timeout,
                wallet,
                host,
                token,
            } => {
                let work = run_upload(file_path, host, &wallet, token);
                let timeout = timeout.unwrap_or(DEFAULT_TIMEOUT);
                match tokio::time::timeout(Duration::from_millis(timeout), work).await {
                    Ok(res) => match res {
                        Ok(ok) => println!("[Ok] {}", ok),
                        Err(err) => println!("[Err] {}", err),
                    },
                    Err(err) => println!("Error running task: {}", err),
                }
            }
            Command::UploadDir {} => todo!(),
            Command::Price {
                byte_amount,
                timeout,
                host,
                token,
            } => {
                let byte_amount = byte_amount.unwrap_or(DEFAULT_BYTE_AMOUNT);
                let work = run_price(host, token, byte_amount);
                let timeout = timeout.unwrap_or(DEFAULT_TIMEOUT);
                match tokio::time::timeout(Duration::from_millis(timeout), work).await {
                    Ok(res) => match res {
                        Ok(ok) => println!("[Ok] {}", ok),
                        Err(err) => println!("[Err] {}", err),
                    },
                    Err(err) => println!("Error running task: {}", err),
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    args.command.execute().await;
}
