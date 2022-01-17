# Bundlr Rust SDK (WIP)


## Example

```rs
use bundlr_sdk::{Bundlr, SolanaSigner, tags::Tag};

#[tokio::main]
async fn main() {
    let signer = SolanaSigner::from_base58("key");
    let bundlr = Bundlr::new(
        "https://node1.bundlr.network".to_string(),
        "solana".to_string(),
        "sol".to_string(),
        signer
    );
    let tx = bundlr.create_transaction_with_tags("hello".into(), vec![Tag::new("hello".into(), "world".into())]);

    // Will return Err if not success
    let response = bundlr.send_transaction(tx).await.unwrap();
}
```