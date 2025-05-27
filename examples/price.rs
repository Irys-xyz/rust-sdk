use irys_sdk::{bundler::get_price, token::TokenType};
use reqwest::Url;

#[tokio::main]
async fn main() {
    let url = Url::parse("https://uploader.irys.xyz").unwrap();
    let token = TokenType::Solana;

    let client = reqwest::Client::new();
    let res = get_price(&url, token, &client, 256000).await;
    match res {
        Ok(ok) => println!("[ok] {}", ok),
        Err(err) => println!("[err] {}", err),
    }
}
