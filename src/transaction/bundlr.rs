use async_stream::try_stream;
use bytes::{BufMut, Bytes};
use futures::Stream;
use ring::rand::SecureRandom;
use std::cmp;
use std::fs::File;
use std::pin::Pin;

use crate::consts::{CHUNK_SIZE, DATAITEM_AS_BUFFER, ONE_AS_BUFFER};
use crate::deep_hash::{deep_hash, DeepHashChunk};
use crate::deep_hash_sync::deep_hash_sync;
use crate::error::BundlrError;
use crate::index::{Config, SignerMap};
use crate::signers::Signer;
use crate::tags::{AvroDecode, AvroEncode, Tag};
use crate::utils::read_offset;

enum Data {
    None,
    Bytes(Vec<u8>),
    Stream(Pin<Box<dyn Stream<Item = anyhow::Result<Bytes>>>>),
}

pub struct BundlrTx {
    signature_type: SignerMap,
    signature: Vec<u8>,
    owner: Vec<u8>,
    target: Vec<u8>,
    anchor: Vec<u8>,
    tags: Vec<Tag>,
    data: Data,
}

impl BundlrTx {
    pub fn new(target: Vec<u8>, data: Vec<u8>, tags: Vec<Tag>) -> Result<Self, BundlrError> {
        let mut randoms: [u8; 32] = [0; 32];
        let sr = ring::rand::SystemRandom::new();
        match sr.fill(&mut randoms) {
            Ok(()) => (),
            Err(err) => return Err(BundlrError::Unknown(err.to_string())),
        }
        let anchor = randoms.to_vec();

        Ok(BundlrTx {
            signature_type: SignerMap::None,
            signature: vec![],
            owner: vec![],
            target,
            anchor,
            tags,
            data: Data::Bytes(data),
        })
    }

    fn from_info_bytes(buffer: &[u8]) -> Result<(Self, usize), BundlrError> {
        let sig_type_b = &buffer[0..2];
        let signature_type = u16::from_le_bytes(
            <[u8; 2]>::try_from(sig_type_b)
                .map_err(|err| BundlrError::BytesError(err.to_string()))?,
        );
        let signer = SignerMap::from(signature_type);

        let Config {
            pub_length,
            sig_length,
            ..
        } = signer.get_config();

        let signature = &buffer[2..2 + sig_length];
        let owner = &buffer[2 + sig_length..2 + sig_length + pub_length];

        let target_start = 2 + sig_length + pub_length;
        let target_present = u8::from_le_bytes(
            <[u8; 1]>::try_from(&buffer[target_start..target_start + 1])
                .map_err(|err| BundlrError::BytesError(err.to_string()))?,
        );
        let target = match target_present {
            0 => &[],
            1 => &buffer[target_start + 1..target_start + 33],
            b => return Err(BundlrError::InvalidPresenceByte(b.to_string())),
        };
        let anchor_start = target_start + 1 + target.len();
        let anchor_present = u8::from_le_bytes(
            <[u8; 1]>::try_from(&buffer[anchor_start..anchor_start + 1])
                .map_err(|err| BundlrError::BytesError(err.to_string()))?,
        );
        let anchor = match anchor_present {
            0 => &[],
            1 => &buffer[anchor_start + 1..anchor_start + 33],
            b => return Err(BundlrError::InvalidPresenceByte(b.to_string())),
        };

        let tags_start = anchor_start + 1 + anchor.len();
        let number_of_tags = u64::from_le_bytes(
            <[u8; 8]>::try_from(&buffer[tags_start..tags_start + 8])
                .map_err(|err| BundlrError::BytesError(err.to_string()))?,
        );

        let number_of_tags_bytes = u64::from_le_bytes(
            <[u8; 8]>::try_from(&buffer[tags_start + 8..tags_start + 16])
                .map_err(|err| BundlrError::BytesError(err.to_string()))?,
        );

        let mut b = buffer.to_vec();
        let mut tags_bytes =
            &mut b[tags_start + 16..tags_start + 16 + number_of_tags_bytes as usize];

        let tags = if number_of_tags_bytes > 0 {
            tags_bytes.decode()?
        } else {
            vec![]
        };

        if number_of_tags != tags.len() as u64 {
            return Err(BundlrError::InvalidTagEncoding);
        }

        let bundlr_tx = BundlrTx {
            signature_type: signer,
            signature: signature.to_vec(),
            owner: owner.to_vec(),
            target: target.to_vec(),
            anchor: anchor.to_vec(),
            tags,
            data: Data::None,
        };

        Ok((bundlr_tx, tags_start + 16 + number_of_tags_bytes as usize))
    }

    pub fn from_bytes(buffer: Vec<u8>) -> Result<Self, BundlrError> {
        let (bundlr_tx, data_start) = BundlrTx::from_info_bytes(&buffer)?;
        let data = &buffer[data_start..buffer.len()];

        Ok(BundlrTx {
            data: Data::Bytes(data.to_vec()),
            ..bundlr_tx
        })
    }

    pub fn from_file_position(
        file: &mut File,
        size: u64,
        offset: u64,
        length: usize,
    ) -> Result<Self, BundlrError> {
        let buffer = read_offset(file, offset, length).map_err(BundlrError::IoError)?;
        let (bundlr_tx, data_start) = BundlrTx::from_info_bytes(&buffer)?;

        let data_start = data_start as u64;
        let data_size = size - data_start;
        let mut file_clone = file.try_clone()?;
        let file_stream = try_stream! {
            let chunk_size = CHUNK_SIZE;
            let mut read = 0;
            while read < data_size {
                let b = read_offset(&mut file_clone, offset + data_start + read, cmp::min(data_size - read, chunk_size) as usize)?;
                read += b.len() as u64;
                yield b;
            };
        };

        Ok(BundlrTx {
            data: Data::Stream(Box::pin(file_stream)),
            ..bundlr_tx
        })
    }

    pub fn is_signed(&self) -> bool {
        !self.signature.is_empty() && self.signature_type != SignerMap::None
    }

    pub fn as_bytes(self) -> Result<Vec<u8>, BundlrError> {
        if !self.is_signed() {
            return Err(BundlrError::NoSignature);
        }
        let data = match &self.data {
            Data::Stream(_) => return Err(BundlrError::InvalidDataType),
            Data::None => return Err(BundlrError::InvalidDataType),
            Data::Bytes(data) => data,
        };

        let encoded_tags = if !self.tags.is_empty() {
            self.tags.encode()?
        } else {
            Bytes::default()
        };
        let config = self.signature_type.get_config();
        let length = 2u64
            + config.sig_length as u64
            + config.pub_length as u64
            + 34
            + 16
            + encoded_tags.len() as u64
            + data.len() as u64;

        let mut b = Vec::with_capacity(
            TryInto::<usize>::try_into(length)
                .map_err(|err| BundlrError::TypeParseError(err.to_string()))?,
        );

        let sig_type: [u8; 2] = (self.signature_type as u16).to_le_bytes();
        let target_presence_byte = if self.target.is_empty() {
            &[0u8]
        } else {
            &[1u8]
        };
        let anchor_presence_byte = if self.anchor.is_empty() {
            &[0u8]
        } else {
            &[1u8]
        };
        b.put(&sig_type[..]);
        b.put(&self.signature[..]);
        b.put(&self.owner[..]);
        b.put(&target_presence_byte[..]);
        b.put(&self.target[..]);
        b.put(&anchor_presence_byte[..]);
        b.put(&self.anchor[..]);
        let number_of_tags = (self.tags.len() as u64).to_le_bytes();
        let number_of_tags_bytes = (encoded_tags.len() as u64).to_le_bytes();
        b.put(number_of_tags.as_slice());
        b.put(number_of_tags_bytes.as_slice());
        if !number_of_tags_bytes.is_empty() {
            b.put(encoded_tags);
        }

        b.put(&data[..]);
        Ok(b)
    }

    pub fn as_byte_stream(
        self,
    ) -> Result<Pin<Box<dyn Stream<Item = anyhow::Result<Bytes>>>>, BundlrError> {
        todo!();
    }

    async fn get_message(&mut self) -> Result<Bytes, BundlrError> {
        let encoded_tags = if !self.tags.is_empty() {
            self.tags.encode()?
        } else {
            Bytes::default()
        };

        match &mut self.data {
            Data::None => Ok(Bytes::new()),
            Data::Bytes(data) => {
                let data_chunk = DeepHashChunk::Chunk(data.clone().into());
                let sig_type = &self.signature_type;
                let sig_type_bytes = sig_type.as_u16().to_string().as_bytes().to_vec();
                deep_hash_sync(DeepHashChunk::Chunks(vec![
                    DeepHashChunk::Chunk(DATAITEM_AS_BUFFER.into()),
                    DeepHashChunk::Chunk(ONE_AS_BUFFER.into()),
                    DeepHashChunk::Chunk(sig_type_bytes.to_vec().into()),
                    DeepHashChunk::Chunk(self.owner.to_vec().into()),
                    DeepHashChunk::Chunk(self.target.to_vec().into()),
                    DeepHashChunk::Chunk(self.anchor.to_vec().into()),
                    DeepHashChunk::Chunk(encoded_tags.clone()),
                    data_chunk,
                ]))
            }
            Data::Stream(file_stream) => {
                let data_chunk = DeepHashChunk::Stream(file_stream);
                let sig_type = &self.signature_type;
                let sig_type_bytes = sig_type.as_u16().to_string().as_bytes().to_vec();
                deep_hash(DeepHashChunk::Chunks(vec![
                    DeepHashChunk::Chunk(DATAITEM_AS_BUFFER.into()),
                    DeepHashChunk::Chunk(ONE_AS_BUFFER.into()),
                    DeepHashChunk::Chunk(sig_type_bytes.to_vec().into()),
                    DeepHashChunk::Chunk(self.owner.to_vec().into()),
                    DeepHashChunk::Chunk(self.target.to_vec().into()),
                    DeepHashChunk::Chunk(self.anchor.to_vec().into()),
                    DeepHashChunk::Chunk(encoded_tags.clone()),
                    data_chunk,
                ]))
                .await
            }
        }
    }

    pub async fn sign(&mut self, signer: &dyn Signer) -> Result<(), BundlrError> {
        self.signature_type = signer.sig_type();
        self.owner = signer.pub_key().to_vec();

        let message = self.get_message().await?;

        let sig = signer.sign(message)?;
        self.signature = sig.to_vec();

        Ok(())
    }

    pub async fn verify(&mut self) -> Result<(), BundlrError> {
        let message = self.get_message().await?;
        let pub_key = &self.owner;
        let signature = &self.signature;

        let verifier = &self.signature_type;
        verifier.verify(pub_key, &message, signature)
    }

    pub fn get_signarure(&self) -> Vec<u8> {
        self.signature.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::tags::Tag;
    #[cfg(feature = "solana")]
    use crate::transaction::bundlr::BundlrTx;
    use crate::{ArweaveSigner, Ed25519Signer, Secp256k1Signer};
    use secp256k1::SecretKey;
    use std::path::PathBuf;
    use std::str::FromStr;
    use std::{fs, fs::File, io::Write};

    #[allow(unused)]
    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }
    #[tokio::test]
    async fn test_create_sign_verify_load_ed25519() {
        let path = "./res/test_bundles/test_data_item_ed25519";
        let secret_key = "kNykCXNxgePDjFbDWjPNvXQRa8U12Ywc19dFVaQ7tebUj3m7H4sF4KKdJwM7yxxb3rqxchdjezX9Szh8bLcQAjb";
        let signer = Ed25519Signer::from_base58(secret_key).unwrap();
        let mut data_item_1 = BundlrTx::new(
            Vec::from(""),
            Vec::from("hello"),
            vec![Tag::new("name", "value")],
        )
        .unwrap();
        let res = data_item_1.sign(&signer).await;
        assert!(res.is_ok());

        let mut f = File::create(path).unwrap();
        let data_item_1_bytes = data_item_1.as_bytes().unwrap();
        f.write_all(&data_item_1_bytes).unwrap();

        let buffer = fs::read(path).expect("Could not read file");
        let data_item_2 = BundlrTx::from_bytes(buffer).expect("Invalid bytes");
        assert!(&data_item_2.is_signed());

        assert_eq!(data_item_1_bytes, data_item_2.as_bytes().unwrap());
    }

    #[tokio::test]
    async fn test_create_sign_verify_load_rsa4096() {
        let path = "./res/test_bundles/test_data_item_rsa4096";
        let key_path = PathBuf::from_str("res/test_wallet.json").unwrap();
        let signer = ArweaveSigner::from_keypair_path(key_path).unwrap();
        let mut data_item_1 = BundlrTx::new(
            Vec::from(""),
            Vec::from("hello"),
            vec![Tag::new("name", "value")],
        )
        .unwrap();
        let res = data_item_1.sign(&signer).await;
        assert!(res.is_ok());

        let mut f = File::create(path).unwrap();
        let data_item_1_bytes = data_item_1.as_bytes().unwrap();
        f.write_all(&data_item_1_bytes).unwrap();

        let buffer = fs::read(path).expect("Could not read file");
        let data_item_2 = BundlrTx::from_bytes(buffer).expect("Invalid bytes");
        assert!(&data_item_2.is_signed());
        assert_eq!(data_item_1_bytes, data_item_2.as_bytes().unwrap());
    }

    #[tokio::test]
    async fn test_create_sign_verify_load_cosmos() {
        //TODO: assign cosmos constant then fix this
        /*
        let path = "./res/test_bundles/test_data_item_cosmos";
        let base58_secret_key = "28PmkjeZqLyfRQogb3FU4E1vJh68dXpbojvS2tcPwezZmVQp8zs8ebGmYg1hNRcjX4DkUALf3SkZtytGWPG3vYhs";
        let signer = CosmosSigner::from_base58(base58_secret_key).unwrap();
        let mut data_item_1 = BundlrTx::new(
            Vec::from(""),
            Vec::from("hello"),
            vec![Tag::new("name", "value")],
        );
        let res = data_item_1.sign(&signer).await;
        assert!(res.is_ok());

        let mut f = File::create(path).unwrap();
        let data_item_1_bytes = data_item_1.as_bytes().unwrap();
        f.write_all(&data_item_1_bytes).unwrap();

        let buffer = fs::read(path).expect("Could not read file");
        let data_item_2 = BundlrTx::from_bytes(buffer).expect("Invalid bytes");
        assert!(&data_item_2.is_signed());
        assert_eq!(data_item_1_bytes, data_item_2.as_bytes().unwrap());
         */
    }

    #[tokio::test]
    async fn test_create_sign_verify_load_secp256k1() {
        let path = "./res/test_bundles/test_data_item_secp256k1";
        let secret_key = SecretKey::from_slice(b"00000000000000000000000000000000").unwrap();
        let signer = Secp256k1Signer::new(secret_key);
        let mut data_item_1 = BundlrTx::new(
            Vec::from(""),
            Vec::from("hello"),
            vec![Tag::new("name", "value")],
        )
        .unwrap();
        let res = data_item_1.sign(&signer).await;
        assert!(res.is_ok());
        let mut f = File::create(path).unwrap();
        let data_item_1_bytes = data_item_1.as_bytes().unwrap();
        f.write_all(&data_item_1_bytes).unwrap();

        let buffer = fs::read(path).expect("Could not read file");
        let data_item_2 = BundlrTx::from_bytes(buffer).expect("Invalid bytes");
        assert!(&data_item_2.is_signed());
        assert_eq!(data_item_1_bytes, data_item_2.as_bytes().unwrap());
    }
}
