use bundlr_sdk::{currency::CurrencyType, Bundlr};
use reqwest::Url;

#[tokio::main]
async fn main() {
    let url = Url::parse("https://node1.bundlr.network").unwrap();
    let currency = CurrencyType::Solana;
    let address = "7y3tfYz8V3ui67XRJi1iiiS5GQ4zVyFoDfFAtouhB8gL";
    let res = Bundlr::get_balance_public(&url, currency, address, &reqwest::Client::new()).await;
    match res {
        Ok(ok) => println!("[ok] {}", ok),
        Err(err) => println!("[err] {}", err),
    }
}
