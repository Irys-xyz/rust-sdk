use data_encoding::BASE64URL_NOPAD;
use irys_sdk::{deep_hash::DeepHashChunk, deep_hash_sync::deep_hash_sync, ArweaveSigner, Verifier};
use serde::{Deserialize, Serialize};

// ===============================================================================================
//  NOTE: this data structure will be included in further versions, along with a verify function.
//  For now, you can verify receipts with following example
// ===============================================================================================

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
    pub id: String,
    pub timestamp: u64,
    pub version: String,
    pub public: String,
    pub signature: String,
    pub deadline_height: u64,
    pub block: u64,
    pub validator_signatures: Vec<String>,
}

fn main() -> Result<(), irys_sdk::error::BundlerError> {
    let data = std::fs::read_to_string("res/test_receipt.json").expect("Unable to read file");
    let receipt = serde_json::from_str::<Receipt>(&data).expect("Unable to parse json file");

    let fields = DeepHashChunk::Chunks(vec![
        DeepHashChunk::Chunk("Bundlr".into()),
        DeepHashChunk::Chunk(receipt.version.into()),
        DeepHashChunk::Chunk(receipt.id.into()),
        DeepHashChunk::Chunk(receipt.deadline_height.to_string().into()),
        DeepHashChunk::Chunk(receipt.timestamp.to_string().into()),
    ]);

    let pubk = BASE64URL_NOPAD
        .decode(&receipt.public.into_bytes())
        .unwrap();
    let msg = deep_hash_sync(fields).unwrap();
    let sig = BASE64URL_NOPAD
        .decode(&receipt.signature.into_bytes())
        .unwrap();

    ArweaveSigner::verify(pubk.into(), msg, sig.into())
}
