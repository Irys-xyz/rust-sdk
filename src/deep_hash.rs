use std::pin::Pin;

use async_recursion::async_recursion;
use bytes::Bytes;
use openssl::sha::Sha384;

use crate::error::BundlrError;
use futures::{Stream, TryStream, TryStreamExt};

const LIST_AS_BUFFER: &[u8] = "list".as_bytes();
const BLOB_AS_BUFFER: &[u8] = "blob".as_bytes();
pub const DATAITEM_AS_BUFFER: &[u8] = "dataitem".as_bytes();
pub const ONE_AS_BUFFER: &[u8] = "1".as_bytes();

pub enum DeepHashChunk {
    Chunk(Bytes),
    Stream(Pin<Box<dyn Stream<Item = anyhow::Result<Bytes>>>>),
    Chunks(Vec<DeepHashChunk>),
}

trait Foo: Stream<Item = anyhow::Result<Bytes>> + TryStream {}

pub async fn deep_hash(chunk: DeepHashChunk) -> Result<Bytes, BundlrError> {
    match chunk {
        DeepHashChunk::Chunk(b) => {
            let tag = [BLOB_AS_BUFFER, b.len().to_string().as_bytes()].concat();
            let c = [sha384hash(tag.into()), sha384hash(b)].concat();
            Ok(Bytes::copy_from_slice(&sha384hash(c.into())))
        }
        DeepHashChunk::Stream(mut s) => {
            let mut hasher = Sha384::new();
            let mut length = 0;
            while let Some(chunk) = s
                .as_mut()
                .try_next()
                .await
                .map_err(|_| BundlrError::NoBytesLeft)?
            {
                length += chunk.len();
                hasher.update(&chunk);
            }

            let tag = [BLOB_AS_BUFFER, length.to_string().as_bytes()].concat();

            let tagged_hash = [
                sha384hash(tag.into()),
                Bytes::copy_from_slice(&hasher.finish()),
            ]
            .concat();

            Ok(sha384hash(tagged_hash.into()))
        }
        DeepHashChunk::Chunks(chunks) => {
            // Be careful of truncation
            let len = chunks.len() as f64;
            let tag = [LIST_AS_BUFFER, len.to_string().as_bytes()].concat();

            let acc = sha384hash(tag.into());

            return deep_hash_chunks(chunks, acc).await;
        }
    }
}

#[async_recursion(?Send)]
pub async fn deep_hash_chunks(
    mut chunks: Vec<DeepHashChunk>,
    acc: Bytes,
) -> Result<Bytes, BundlrError> {
    if chunks.is_empty() {
        return Ok(acc);
    };

    let acc = Bytes::copy_from_slice(&acc);

    let hash_pair = [acc, deep_hash(chunks.remove(0)).await?].concat();
    let new_acc = sha384hash(hash_pair.into());
    deep_hash_chunks(chunks, new_acc).await
}

fn sha384hash(b: Bytes) -> Bytes {
    let mut hasher = Sha384::new();
    hasher.update(&b);
    Bytes::copy_from_slice(&hasher.finish())
}
