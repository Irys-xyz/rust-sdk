use bundlr_sdk::{currency::CurrencyType, Bundlr};
use reqwest::Url;

#[tokio::main]
async fn main() {
    let url = Url::parse("https://node1.bundlr.network").unwrap();
    let currency = CurrencyType::Solana;

    let client = reqwest::Client::new();
    let res = Bundlr::get_price_public(&url, currency, &client, 256000).await;
    match res {
        Ok(ok) => println!("[ok] {}", ok),
        Err(err) => println!("[err] {}", err),
    }
}
