use core::slice::SlicePattern;
use std::{any::TypeId, cmp, ops::Sub, rc::Rc, vec};

use async_stream::stream;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use data_encoding::BASE64URL;
use derive_more::{Display, Error};
use futures::stream::TryStreamExt;
use futures::Stream;
use num_traits::FromPrimitive;
use primitive_types::U256;
use serde::{Deserialize, Serialize};

use crate::{error::BundleError, index::SignerMap, tags::AvroDecode};

async fn verify_and_index_stream(
    mut s: impl Stream<Item = Result<Bytes, anyhow::Error>> + Unpin,
) -> Result<Vec<Item>, BundleError> {
    // Assume average number of items to be 500
    let mut header_bytes = BytesMut::with_capacity(32 + (64 * 500));

    // Read first 32 bytes for item count
    read(&mut header_bytes, 32, &mut s).await?;

    // TODO: Test this for max val
    let length = U256::from_little_endian(&header_bytes[0..32]).as_usize();

    header_bytes.advance(32);

    // Read header bytes
    read(&mut header_bytes, 64 * length, &mut s).await?;

    let mut headers = Vec::with_capacity(cmp::min(length, 1000));

    for i in 0..length {
        let start = 64 * i;
        let size = U256::from_little_endian(&header_bytes[start..(start + 32)]);
        let id = BASE64URL.encode(&header_bytes[(start + 32)..(start + 64)]);
        headers.push(Header(size, id));
    }

    let mut item_bytes = BytesMut::from(&header_bytes[32 + (length * 64)..]);

    // Free header bytes
    drop(header_bytes);

    let mut items = Vec::with_capacity(cmp::min(length, 1000));

    for Header(size, id) in headers {
        // Get sig type
        read(&mut item_bytes, 2, &mut s).await?;
        let signature_type = u16::from_le_bytes(item_bytes[0..2].try_into()?);

        let signer: SignerMap = SignerMap::from_u16(signature_type)?;
        let signer_config = signer.get_config();
        item_bytes.advance(2);

        // Get sig
        read(&mut item_bytes, signer_config.sig_length.into(), &mut s).await?;
        let signature = &item_bytes[..signer_config.sig_length.into()];
        item_bytes.advance(signer_config.sig_length.into());

        // Get pub
        read(&mut item_bytes, signer_config.pub_length.into(), &mut s).await?;
        let public = &item_bytes[..signer_config.pub_length.into()];
        item_bytes.advance(signer_config.pub_length.into());

        // Get tags
        read(&mut item_bytes, 16, &mut s).await?;
        let number_of_tags = u8::from_le_bytes(item_bytes[0..8].try_into()?);
        let number_of_tags_bytes = u16::from_le_bytes(item_bytes[8..16].try_into()?);
        item_bytes.advance(16);

        let tags = (&item_bytes[..number_of_tags_bytes as usize]).decode()?;
        if tags.len() != number_of_tags as usize {
            return Err(BundleError::InvalidTagEncoding);
        }

        let non_data_size = 2 + signer_config.total_length() + 16 + number_of_tags_bytes as u32;
        item_bytes.advance(non_data_size.try_into()?);

        let data_size = size.sub(non_data_size);

        let data_stream = stream! {
            let data_count = U256::zero();
            while (data_count < data_size) {
                match s.try_next().await.map_err(|_| BundleError::NoBytesLeft)? {
                    Some(b) => yield Ok(b),
                    None => {
                        yield Err(BundleError::NoBytesLeft);
                        return ();
                    }
                };
            };

            if data_size > data_count {
                println!("{}", "Bad sizes");
            };

            item_bytes.advance((data_count - data_size).as_usize());
        };

        let item = Item {
            id: "id".to_string(),
        };

        items.push(item);
    }

    Ok(vec![])
}

async fn read(
    b: &mut BytesMut,
    len: usize,
    mut s: impl Stream<Item = Result<Bytes, anyhow::Error>> + Unpin,
) -> Result<(), BundleError> {
    if b.len() >= len {
        return Ok(());
    };

    while b.len() < len {
        let next = &s.try_next().await;
        let new_bytes = match next.as_ref().map_err(|_| BundleError::NoBytesLeft)? {
            Some(bytess) => bytess,
            None => return Err(BundleError::NoBytesLeft),
        };

        b.extend(new_bytes);
    }

    Ok(())
}

async fn produce_data_stream(
    mut s: impl Stream<Item = Result<Bytes, anyhow::Error>> + Unpin,
) -> Result<(), BundleError> {
}

#[cfg(test)]
mod tests {
    use crate::stream::verify_and_index_stream;

    #[actix_web::test]
    async fn test() {
        // let client = awc::Client::default();
        // let stream = client
        //         .get("https://google.com")
        //         .send()
        //         .await.unwrap();

        // assert!(verify_and_index_stream(stream).await.is_err());
    }
}
