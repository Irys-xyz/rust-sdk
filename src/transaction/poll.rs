use std::{thread::sleep, time::Duration};

use crate::currency::Currency;

/// Number of seconds to wait between retying to post a failed chunk.
pub const RETRY_SLEEP: u64 = 10;

/// Number of confirmations needed to consider a transaction funded
pub const CONFIRMATIONS_NEEDED: u64 = 5;

pub struct ConfirmationPoll();

impl ConfirmationPoll {
    pub async fn await_confirmation(tx_id: &String, currency: &dyn Currency) {
        let mut confirmations = 0;
        while confirmations < CONFIRMATIONS_NEEDED {
            let (status, tx_status) = currency
                .get_tx_status(tx_id.to_string())
                .await
                .expect("Could not get tx status");

            if let Some(tx_status) = tx_status {
                confirmations = tx_status.confirmations
            }

            sleep(Duration::from_secs(RETRY_SLEEP));
        }
    }
}
