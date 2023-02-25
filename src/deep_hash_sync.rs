use bytes::Bytes;
use sha2::{Digest, Sha384};

use crate::{
    consts::{BLOB_AS_BUFFER, LIST_AS_BUFFER},
    deep_hash::DeepHashChunk,
    error::BundlrError,
};
use futures::{Stream, TryStream};

trait Foo: Stream<Item = anyhow::Result<Bytes>> + TryStream {}

pub fn deep_hash_sync(chunk: DeepHashChunk) -> Result<Bytes, BundlrError> {
    match chunk {
        DeepHashChunk::Chunk(b) => {
            let tag = [BLOB_AS_BUFFER, b.len().to_string().as_bytes()].concat();
            let c = [sha384hash(tag.into()), sha384hash(b)].concat();
            Ok(Bytes::copy_from_slice(&sha384hash(c.into())))
        }
        DeepHashChunk::Chunks(chunks) => {
            // Be careful of truncation
            let len = chunks.len() as f64;
            let tag = [LIST_AS_BUFFER, len.to_string().as_bytes()].concat();

            let acc = sha384hash(tag.into());

            deep_hash_chunks_sync(chunks, acc)
        }
        _ => Err(BundlrError::Unsupported(
            "Streaming is not supported for sync".to_owned(),
        )),
    }
}

pub fn deep_hash_chunks_sync(
    mut chunks: Vec<DeepHashChunk>,
    acc: Bytes,
) -> Result<Bytes, BundlrError> {
    if chunks.is_empty() {
        return Ok(acc);
    };

    let acc = Bytes::copy_from_slice(&acc);

    let hash_pair = [acc, deep_hash_sync(chunks.remove(0))?].concat();
    let new_acc = sha384hash(hash_pair.into());
    deep_hash_chunks_sync(chunks, new_acc)
}

fn sha384hash(b: Bytes) -> Bytes {
    let mut hasher = Sha384::new();
    hasher.update(&b);
    Bytes::copy_from_slice(&hasher.finalize())
}
