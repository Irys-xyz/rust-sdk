use num::BigUint;

pub mod bundlr;
pub mod poll;

pub struct Tx {
    from: String,
    to: String,
    amount: BigUint,
    block_height: BigUint,
    pending: bool,
    confirmed: bool,
}
