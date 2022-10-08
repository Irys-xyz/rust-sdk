use crate::currency::Currency;

pub struct ConfirmationPoll {
    _tx: String,
}

impl ConfirmationPoll {
    pub async fn await_confirmation(tx_id: &String, currency: &dyn Currency) -> bool {
        todo!()
    }
}
