use num::BigUint;

pub mod bundlr;
pub mod poll;

pub struct Tx {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub block_height: u128,
    pub pending: bool,
    pub confirmed: bool,
}
