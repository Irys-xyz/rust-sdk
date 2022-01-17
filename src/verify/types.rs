use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
    pub id: String,
}

pub struct Header(pub u64, pub String);