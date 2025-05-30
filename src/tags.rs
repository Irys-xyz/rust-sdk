use avro_rs::{from_avro_datum, to_avro_datum, Schema};
use bytes::Bytes;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::error::BundlerError;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Tag {
    pub name: String,
    pub value: String,
}

impl Tag {
    pub fn new(name: &str, value: &str) -> Self {
        Tag {
            name: name.to_string(),
            value: value.to_string(),
        }
    }
}

const SCHEMA_STR: &str = r#"{
    "type": "array",
    "items": {
        "type": "record",
        "name": "Tag",
        "fields": [
            { "name": "name", "type": "string" },
            { "name": "value", "type": "string" }
        ]
    }
}"#;

lazy_static! {
    pub static ref TAGS_SCHEMA: Schema = Schema::parse_str(SCHEMA_STR).unwrap();
}

// const TAGS_READER: Reader<'static, Vec<Tag>> = Reader::with_schema(&TAGS_SCHEMA, Vec::<Tag>::new());
// const TAGS_WRITER: Writer<'static, Vec<Tag>> = Writer::new(&TAGS_SCHEMA, Vec::new());

pub trait AvroEncode {
    fn encode(&self) -> Result<Bytes, BundlerError>;
}

pub trait AvroDecode {
    fn decode(&mut self) -> Result<Vec<Tag>, BundlerError>;
}

impl AvroEncode for Vec<Tag> {
    fn encode(&self) -> Result<Bytes, BundlerError> {
        let v = avro_rs::to_value(self)?;
        to_avro_datum(&TAGS_SCHEMA, v)
            .map(|v| v.into())
            .map_err(|_| BundlerError::NoBytesLeft)
    }
}

impl AvroDecode for &mut [u8] {
    fn decode(&mut self) -> Result<Vec<Tag>, BundlerError> {
        let x = self.to_vec();
        let v = from_avro_datum(&TAGS_SCHEMA, &mut x.as_slice(), Some(&TAGS_SCHEMA))
            .map_err(|_| BundlerError::InvalidTagEncoding)?;
        avro_rs::from_value(&v).map_err(|_| BundlerError::InvalidTagEncoding)
    }
}

impl From<avro_rs::DeError> for BundlerError {
    fn from(_: avro_rs::DeError) -> Self {
        BundlerError::InvalidTagEncoding
    }
}

#[cfg(test)]
mod tests {

    use crate::tags::{AvroDecode, AvroEncode};

    use super::Tag;

    #[test]
    fn test_bytes() {
        let b = &[2u8, 8, 110, 97, 109, 101, 10, 118, 97, 108, 117, 101, 0];

        let mut sli = &mut b.clone()[..];

        dbg!((sli).decode()).unwrap();
    }

    #[test]
    fn test_tags() {
        let tags = vec![Tag {
            name: "name".to_string(),
            value: "value".to_string(),
        }];

        dbg!(tags.encode().unwrap().to_vec());
    }
}
