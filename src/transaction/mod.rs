pub mod bundlr;
pub mod poll;

#[derive(Debug)]
pub struct TxStatus {
    pub confirmations: u64,
    pub height: u128,
    pub block_hash: String,
}

pub struct Tx {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub fee: u64,
    pub block_height: u128,
    pub pending: bool,
    pub confirmed: bool,
}
