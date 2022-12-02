use std::pin::Pin;

use async_recursion::async_recursion;
use bytes::Bytes;
use sha2::{Digest, Sha384};

use crate::{
    consts::{BLOB_AS_BUFFER, LIST_AS_BUFFER},
    error::BundlrError,
};
use futures::{Stream, TryStream, TryStreamExt};

pub enum DeepHashChunk<'a> {
    Chunk(Bytes),
    Stream(&'a mut Pin<Box<dyn Stream<Item = anyhow::Result<Bytes>>>>),
    Chunks(Vec<DeepHashChunk<'a>>),
}

trait Foo: Stream<Item = anyhow::Result<Bytes>> + TryStream {}

pub async fn deep_hash(chunk: DeepHashChunk<'_>) -> Result<Bytes, BundlrError> {
    match chunk {
        DeepHashChunk::Chunk(b) => {
            let tag = [BLOB_AS_BUFFER, b.len().to_string().as_bytes()].concat();
            let c = [sha384hash(tag.into()), sha384hash(b)].concat();
            Ok(Bytes::copy_from_slice(&sha384hash(c.into())))
        }
        DeepHashChunk::Stream(s) => {
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
                Bytes::copy_from_slice(&hasher.finalize()),
            ]
            .concat();

            Ok(sha384hash(tagged_hash.into()))
        }
        DeepHashChunk::Chunks(mut chunks) => {
            // Be careful of truncation
            let len = chunks.len() as f64;
            let tag = [LIST_AS_BUFFER, len.to_string().as_bytes()].concat();

            let acc = sha384hash(tag.into());

            deep_hash_chunks(&mut chunks, acc).await
        }
    }
}

#[async_recursion(?Send)]
pub async fn deep_hash_chunks(
    chunks: &mut Vec<DeepHashChunk<'_>>,
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
    Bytes::copy_from_slice(&hasher.finalize())
}
