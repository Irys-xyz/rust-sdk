use super::types::{Header, Item};
use crate::consts::CHUNK_SIZE;
use crate::tags::AvroDecode;
use crate::utils::read_offset;
use crate::{
    deep_hash::{deep_hash, DeepHashChunk, DATAITEM_AS_BUFFER, ONE_AS_BUFFER},
    error::BundlrError,
    index::{Config, SignerMap},
};
use async_stream::try_stream;
use data_encoding::BASE64URL;
use futures::stream;
use num_traits::FromPrimitive;
use primitive_types::U256;
use std::{cmp, fs::File};

impl From<std::io::Error> for BundlrError {
    fn from(e: std::io::Error) -> Self {
        BundlrError::FsError(e.to_string())
    }
}

pub async fn verify_file_bundle(filename: String) -> Result<Vec<Item>, BundlrError> {
    let mut file = File::open(&filename).unwrap();

    let bundle_length = U256::from_little_endian(&read_offset(&mut file, 0, 32)?).as_u64();

    // NOTE THIS IS UNSAFE BEYOND USIZE LIMIT
    let header_bytes = read_offset(&mut file, 32, bundle_length as usize * 64)?;
    // This will use ~100 bytes per header. So 1 GB is 1e+7 headers
    let mut headers = Vec::with_capacity(cmp::min(bundle_length as usize, 1000));

    for i in (0..(64 * usize::try_from(bundle_length).unwrap())).step_by(64) {
        let h = Header(
            U256::from_little_endian(&header_bytes[i..i + 32]).as_u64(),
            BASE64URL.encode(&header_bytes[i + 32..i + 64]),
        );
        headers.push(h);
    }

    let mut offset = 32 + (64 * bundle_length);
    let mut items = Vec::with_capacity(cmp::min(bundle_length as usize, 1000));

    for Header(size, id) in headers {
        // Read 4 KiB - max data-less Bundlr tx
        // We do it all at once to improve performance - by lowering fs ops and doing ops in memory
        let buffer = read_offset(&mut file, offset, 4096)?;

        let sig_type_b = &buffer[0..2];
        let sig_type = u16::from_le_bytes(<[u8; 2]>::try_from(sig_type_b).unwrap());
        let signer: SignerMap = match SignerMap::from_u16(sig_type) {
            Some(s) => s,
            None => return Err(BundlrError::InvalidSignerType),
        };
        let Config {
            pub_length,
            sig_length,
        } = signer.get_config();

        let sig = &buffer[2..2 + sig_length];
        dbg!(sig);

        let pub_key = &buffer[2 + sig_length..2 + sig_length + pub_length];

        let target_start = 2 + sig_length + pub_length;
        let target_present = u8::from_le_bytes(
            <[u8; 1]>::try_from(&buffer[target_start..target_start + 1]).unwrap(),
        );
        let target = match target_present {
            0 => &[],
            1 => &buffer[target_start + 1..target_start + 33],
            b => return Err(BundlrError::InvalidPresenceByte(b.to_string())),
        };
        let anchor_start = target_start + 1 + target.len();
        let anchor_present = u8::from_le_bytes(
            <[u8; 1]>::try_from(&buffer[anchor_start..anchor_start + 1]).unwrap(),
        );
        let anchor = match anchor_present {
            0 => &[],
            1 => &buffer[anchor_start + 1..anchor_start + 33],
            b => return Err(BundlrError::InvalidPresenceByte(b.to_string())),
        };

        let tags_start = anchor_start + 1 + anchor.len();
        let number_of_tags =
            u64::from_le_bytes(<[u8; 8]>::try_from(&buffer[tags_start..tags_start + 8]).unwrap());

        let number_of_tags_bytes = u64::from_le_bytes(
            <[u8; 8]>::try_from(&buffer[tags_start + 8..tags_start + 16]).unwrap(),
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

        let data_start = tags_start as u64 + 16 + number_of_tags_bytes;
        let data_size = size - data_start;

        let mut file_clone = file.try_clone().unwrap();
        let file_stream = try_stream! {
            let chunk_size = CHUNK_SIZE;
            let mut read = 0;
            while read < data_size {
                let b = read_offset(&mut file_clone, offset + data_start + read, cmp::min(data_size - read, chunk_size) as usize).unwrap();
                read += b.len() as u64;
                yield b;
            };
        };

        let e_sig_type = sig_type.to_string().as_bytes().to_vec();

        let message = deep_hash(DeepHashChunk::Chunks(vec![
            DeepHashChunk::Chunk(DATAITEM_AS_BUFFER.into()),
            DeepHashChunk::Chunk(ONE_AS_BUFFER.into()),
            DeepHashChunk::Chunk(e_sig_type.into()),
            DeepHashChunk::Chunk(pub_key.to_vec().into()),
            DeepHashChunk::Chunk(target.to_vec().into()),
            DeepHashChunk::Chunk(anchor.to_vec().into()),
            DeepHashChunk::Chunk(tags_bytes.to_vec().into()),
            DeepHashChunk::Stream(Box::pin(file_stream)),
        ]))
        .await?;

        if !signer.verify(pub_key, &message, sig)? {
            return Err(BundlrError::InvalidSignature);
        };

        let item = Item {
            tx_id: id,
            signature: sig.to_vec(),
        };

        items.push(item);

        offset += size;
    }

    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::verify_file_bundle;

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e).unwrap()
        };
    }

    #[test]
    fn should_verify_test_bundle() {
        println!(
            "{:?}",
            aw!(verify_file_bundle(
                "./src/verify/test_bundles/test_bundle".to_string()
            ))
        );
        assert_eq!(1, 1)
    }

    #[test]
    fn should_verify_arweave() {
        println!(
            "{:?}",
            aw!(verify_file_bundle(
                "./src/verify/test_bundles/arweave_sig".to_string()
            ))
        );

        assert_eq!(1, 1)
    }

    #[test]
    #[cfg(any(feature = "ethereum", feature = "erc20"))]
    fn should_verify_secp256k1() {
        println!(
            "{:?}",
            aw!(verify_file_bundle(
                "./src/verify/test_bundles/ethereum_sig".to_string()
            ))
        );
        println!(
            "{:?}",
            aw!(verify_file_bundle(
                "./src/verify/test_bundles/arbitrum_sig".to_string()
            ))
        );
        println!(
            "{:?}",
            aw!(verify_file_bundle(
                "./src/verify/test_bundles/avalanche_sig".to_string()
            ))
        );
        println!(
            "{:?}",
            aw!(verify_file_bundle(
                "./src/verify/test_bundles/bnb_sig".to_string()
            ))
        );
        println!(
            "{:?}",
            aw!(verify_file_bundle(
                "./src/verify/test_bundles/boba-eth_sig".to_string()
            ))
        );
        println!(
            "{:?}",
            aw!(verify_file_bundle(
                "./src/verify/test_bundles/chainlink_sig".to_string()
            ))
        );
        println!(
            "{:?}",
            aw!(verify_file_bundle(
                "./src/verify/test_bundles/kyve_sig".to_string()
            ))
        );
        println!(
            "{:?}",
            aw!(verify_file_bundle(
                "./src/verify/test_bundles/matic_sig".to_string()
            ))
        );
    }

    #[test]
    #[cfg(feature = "cosmos")]
    fn should_verify_cosmos() {
        println!(
            "{:?}",
            aw!(verify_file_bundle(
                "./src/verify/test_bundles/cosmos_sig".to_string()
            ))
        );
        assert_eq!(1, 1)
    }

    #[test]
    #[cfg(any(feature = "solana", feature = "algorand"))]
    fn should_verify_ed25519() {
        println!(
            "{:?}",
            aw!(verify_file_bundle(
                "./src/verify/test_bundles/solana_sig".to_string()
            ))
        );
        println!(
            "{:?}",
            aw!(verify_file_bundle(
                "./src/verify/test_bundles/algorand_sig".to_string()
            ))
        );
        println!(
            "{:?}",
            aw!(verify_file_bundle(
                "./src/verify/test_bundles/near_sig".to_string()
            ))
        );
    }
}
