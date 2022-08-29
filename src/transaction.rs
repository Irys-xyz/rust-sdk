use bytes::{BufMut, Bytes};
use ring::rand::SecureRandom;

use crate::deep_hash::{DeepHashChunk, DATAITEM_AS_BUFFER, ONE_AS_BUFFER};
use crate::deep_hash_sync::deep_hash_sync;
use crate::signers::signer::Signer;
use crate::tags::{AvroEncode, Tag};

pub struct BundlrTx(Vec<u8>);

impl BundlrTx {
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }

    pub fn create_with_tags(data: Vec<u8>, tags: Vec<Tag>, signer: &dyn Signer) -> Self {
        let encoded_tags = if !tags.is_empty() {
            tags.encode().unwrap()
        } else {
            Bytes::default()
        };
        let length = 2u64
            + signer.get_sig_length() as u64
            + signer.get_pub_length() as u64
            + 34
            + 16
            + encoded_tags.len() as u64
            + data.len() as u64;
        let mut b = Vec::with_capacity(length.try_into().unwrap());

        let mut randoms: [u8; 32] = [0; 32];
        let sr = ring::rand::SystemRandom::new();
        sr.fill(&mut randoms).unwrap();

        let anchor = Bytes::copy_from_slice(&randoms[..]);
        // let sr = ring::rand::SystemRandom::new();
        // sr.fill(&mut anchor).unwrap();

        let sig_type = signer.sig_type();

        let sig_type_bytes = sig_type.to_string().as_bytes().to_vec();

        let message = deep_hash_sync(DeepHashChunk::Chunks(vec![
            DeepHashChunk::Chunk(DATAITEM_AS_BUFFER.into()),
            DeepHashChunk::Chunk(ONE_AS_BUFFER.into()),
            DeepHashChunk::Chunk(Bytes::copy_from_slice(&sig_type_bytes[..])),
            DeepHashChunk::Chunk(signer.pub_key()),
            DeepHashChunk::Chunk(Bytes::default()),
            DeepHashChunk::Chunk(Bytes::copy_from_slice(&anchor[..])),
            DeepHashChunk::Chunk(encoded_tags.clone()),
            DeepHashChunk::Chunk(data.clone().into()),
        ]))
        .unwrap();

        let sig = signer.sign(message).unwrap();

        // Put sig type
        let sig_type = (signer.sig_type() as u16).to_le_bytes();
        b.put(&sig_type[..]);

        // Put sig
        b.put(sig);

        // Put owner
        b.put(signer.pub_key());

        // Put target
        let target = &[0u8];
        b.put(target.as_slice());

        // Put anchor
        b.put(&[1u8][..]);
        b.put(&anchor[..]);

        // Put tags
        let number_of_tags = (tags.len() as u64).to_le_bytes();
        let number_of_tags_bytes = (encoded_tags.len() as u64).to_le_bytes();
        b.put(number_of_tags.as_slice());
        b.put(number_of_tags_bytes.as_slice());
        if !number_of_tags_bytes.is_empty() {
            b.put(encoded_tags);
        }

        // Put data
        b.put(&data[..]);

        BundlrTx(b)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "solana")]
    use crate::Ed25519Signer;
    use crate::{tags::Tag, transaction::BundlrTx};
    use std::{fs::File, io::Write};

    #[allow(unused)]
    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    #[cfg(feature = "solana")]
    fn test_x() {
        let secret_key = "28PmkjeZqLyfRQogb3FU4E1vJh68dXpbojvS2tcPwezZmVQp8zs8ebGmYg1hNRcjX4DkUALf3SkZtytGWPG3vYhs";
        let signer = Ed25519Signer::from_base58(secret_key);
        let data_item = BundlrTx::create_with_tags(
            Vec::from("hello"),
            vec![Tag::new("name".to_string(), "value".to_string())],
            &signer,
        );

        let mut f = File::create("test_item").unwrap();
        f.write_all(&data_item.0).unwrap();
        println!("{}", data_item.0.len());
    }
}
