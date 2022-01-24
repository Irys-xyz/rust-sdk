use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
    pub tx_id: String,
    pub signature: Vec::<u8>
}

pub struct Header(pub u64, pub String);