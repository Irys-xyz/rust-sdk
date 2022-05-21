use bytes::Bytes;
use openssl::sha::Sha384;

use crate::{deep_hash::DeepHashChunk, error::BundlrError};
use futures::{Stream, TryStream};

const LIST_AS_BUFFER: &[u8] = "list".as_bytes();
const BLOB_AS_BUFFER: &[u8] = "blob".as_bytes();
pub const DATAITEM_AS_BUFFER: &[u8] = "dataitem".as_bytes();
pub const ONE_AS_BUFFER: &[u8] = "1".as_bytes();

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
        _ => panic!("Streaming is not supported for sync"),
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
    Bytes::copy_from_slice(&hasher.finish())
}
