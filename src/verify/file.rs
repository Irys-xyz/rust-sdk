use super::types::{Header, Item};
use crate::error::BundlrError;
use crate::utils::read_offset;
use crate::BundlrTx;
use data_encoding::BASE64URL;
use primitive_types::U256;
use std::{cmp, fs::File};

impl From<std::io::Error> for BundlrError {
    fn from(e: std::io::Error) -> Self {
        BundlrError::FsError(e.to_string())
    }
}

pub async fn verify_file_bundle(filename: String) -> Result<Vec<Item>, BundlrError> {
    let mut file = File::open(&filename)?;

    let bundle_length = U256::from_little_endian(&read_offset(&mut file, 0, 32)?).as_u64();

    // NOTE THIS IS UNSAFE BEYOND USIZE LIMIT
    let header_bytes = read_offset(&mut file, 32, bundle_length as usize * 64)?;
    // This will use ~100 bytes per header. So 1 GB is 1e+7 headers
    let mut headers = Vec::with_capacity(cmp::min(bundle_length as usize, 1000));

    for i in (0..(64
        * usize::try_from(bundle_length)
            .map_err(|err| BundlrError::TypeParseError(err.to_string()))?))
        .step_by(64)
    {
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
        let mut tx = BundlrTx::from_file_position(&mut file, size, offset, 4096)?;

        match tx.verify().await {
            Err(err) => return Err(err),
            Ok(_) => {
                let sig = tx.get_signarure();
                let item = Item {
                    tx_id: id,
                    signature: sig,
                };
                items.push(item);
                offset += size;
            }
        }
    }

    Ok(items)
}

#[cfg(test)]
mod tests {
    use crate::error::BundlrError;

    use super::verify_file_bundle;

    #[tokio::test]
    async fn should_verify_test_bundle() -> Result<(), BundlrError> {
        verify_file_bundle("./res/test_bundles/test_bundle".to_string())
            .await
            .map(|_| ())
    }

    #[tokio::test]
    async fn should_verify_arweave() -> Result<(), BundlrError> {
        verify_file_bundle("./res/test_bundles/arweave_sig".to_string())
            .await
            .map(|_| ())
    }

    #[tokio::test]
    async fn should_verify_secp256k1() -> Result<(), BundlrError> {
        verify_file_bundle("./res/test_bundles/ethereum_sig".to_string()).await?;
        verify_file_bundle("./res/test_bundles/typedethereum_sig".to_string()).await?;
        verify_file_bundle("./res/test_bundles/arbitrum_sig".to_string()).await?;
        verify_file_bundle("./res/test_bundles/avalanche_sig".to_string()).await?;
        verify_file_bundle("./res/test_bundles/bnb_sig".to_string()).await?;
        verify_file_bundle("./res/test_bundles/boba-eth_sig".to_string()).await?;
        verify_file_bundle("./res/test_bundles/chainlink_sig".to_string()).await?;
        verify_file_bundle("./res/test_bundles/kyve_sig".to_string()).await?;
        verify_file_bundle("./res/test_bundles/matic_sig".to_string()).await?;
        Ok(())
    }

    /*
    #[tokio::test]
    #[cfg(feature = "cosmos")]
    async fn should_verify_cosmos() {
        //TODO: update cosmos signed transaction when its constant is defined
        assert!(
            verify_file_bundle("./res/test_bundles/cosmos_sig".to_string())
            .await
            .is_ok()
        );
    }
    */

    #[tokio::test]
    async fn should_verify_ed25519() -> Result<(), BundlrError> {
        verify_file_bundle("./res/test_bundles/solana_sig".to_string()).await?;
        verify_file_bundle("./res/test_bundles/algorand_sig".to_string()).await?;
        verify_file_bundle("./res/test_bundles/near_sig".to_string()).await?;
        verify_file_bundle("./res/test_bundles/aptos_sig".to_string()).await?;
        verify_file_bundle("./res/test_bundles/aptos_multisig".to_string()).await?;
        Ok(())
    }

    #[tokio::test]
    async fn should_verify_random_bundles() -> Result<(), BundlrError> {
        for i in 1..100 {
            verify_file_bundle(format!("./res/gen_bundles/bundle_{}", i).to_string()).await?;
        }
        Ok(())
    }
}
