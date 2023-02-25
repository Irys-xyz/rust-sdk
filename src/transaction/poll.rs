use std::{thread::sleep, time::Duration};

use crate::{
    consts::{CONFIRMATIONS_NEEDED, RETRY_SLEEP},
    currency::Currency,
};

pub struct ConfirmationPoll();

#[allow(unused)]
impl ConfirmationPoll {
    pub async fn await_confirmation(tx_id: &String, currency: &dyn Currency) {
        let mut confirmations = 0;
        while confirmations < CONFIRMATIONS_NEEDED {
            let (status, tx_status) = match currency.get_tx_status(tx_id.to_string()).await {
                Ok(ok) => ok,
                Err(err) => continue,
            };

            if let Some(tx_status) = tx_status {
                confirmations = tx_status.confirmations
            }

            sleep(Duration::from_secs(RETRY_SLEEP));
        }
    }
}
