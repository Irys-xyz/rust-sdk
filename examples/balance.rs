use irys_sdk::{bundler::get_balance, token::TokenType};
use reqwest::Url;

#[tokio::main]
async fn main() {
    let url = Url::parse("https://uploader.irys.network").unwrap();
    let token = TokenType::Solana;
    let address = "7y3tfYz8V3ui67XRJi1iiiS5GQ4zVyFoDfFAtouhB8gL";
    let res = get_balance(&url, token, address, &reqwest::Client::new()).await;
    match res {
        Ok(ok) => println!("[ok] {}", ok),
        Err(err) => println!("[err] {}", err),
    }
}
