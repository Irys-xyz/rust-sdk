use std::{thread::sleep, time::Duration};

use crate::{
    consts::{CONFIRMATIONS_NEEDED, RETRY_SLEEP},
    token::Token,
};

pub struct ConfirmationPoll();

#[allow(unused)]
impl ConfirmationPoll {
    pub async fn await_confirmation(tx_id: &String, token: &dyn Token) {
        let mut confirmations = 0;
        while confirmations < CONFIRMATIONS_NEEDED {
            let (status, tx_status) = match token.get_tx_status(tx_id.to_string()).await {
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
