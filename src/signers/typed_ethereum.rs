use crate::{
    error::BundlrError,
    index::SignerMap,
    utils::{hash_structured_data, EIP712},
    Signer, Verifier,
};
use bytes::Bytes;
use secp256k1::constants::COMPACT_SIGNATURE_SIZE;
use serde_json::{from_str, json};
use web3::signing::recover;

pub struct TypedEthereumSigner {
    //signer: Secp256k1Signer,
    //address: Vec<u8>,
}

const SIG_TYPE: SignerMap = SignerMap::Ethereum;
const SIG_LENGTH: u16 = (COMPACT_SIGNATURE_SIZE + 1) as u16;
const PUB_LENGTH: u16 = 42;

impl Signer for TypedEthereumSigner {
    fn pub_key(&self) -> bytes::Bytes {
        todo!();
    }

    fn sign(&self, _message: bytes::Bytes) -> Result<bytes::Bytes, crate::error::BundlrError> {
        todo!();
    }

    fn sig_type(&self) -> SignerMap {
        SIG_TYPE
    }
    fn get_sig_length(&self) -> u16 {
        SIG_LENGTH
    }
    fn get_pub_length(&self) -> u16 {
        PUB_LENGTH
    }
}

impl Verifier for TypedEthereumSigner {
    fn verify(
        public_key: Bytes,
        message: Bytes,
        signature: Bytes,
    ) -> Result<(), crate::error::BundlrError> {
        let address = String::from_utf8(public_key.to_vec()).map_err(|err| {
            BundlrError::ParseError(format!(
                "Error parsing address from bytes to string: {}",
                err
            ))
        })?;

        let mut hex_message: String = "0x".to_owned();
        for i in 0..message.len() {
            let byte = message[i];
            hex_message += &format!("{:02X}", byte);
        }

        let json = json!({
            "primaryType": "Bundlr",
            "domain": {
                "name": "Bundlr",
                "version": "1"
            },
            "types": {
                "EIP712Domain": [
                    { "name": "name", "type": "string" },
                    { "name": "version", "type": "string" }
                ],
                "Bundlr": [
                    { "name": "Transaction hash", "type": "bytes" },
                    { "name": "address", "type": "address" }
                ]
            },
            "message": {
                "address": address,
                "Transaction hash": hex_message
            }
        });

        let typed_data = from_str::<EIP712>(&json.to_string()).map_err(|err| {
            BundlrError::ParseError(format!("Error parsing EIP712 json object: {}", err))
        })?;
        let data = hash_structured_data(typed_data).map_err(BundlrError::Eip712Error)?;
        let recovered_address = recover(&data, &signature[0..64], signature[64] as i32 - 27)
            .map_err(BundlrError::RecoveryError)?;

        // Somehow, recovered_address.to_string() returns 0x0000..0000 instead of full address ¬¬
        let recovered_address = format!("{:?}", recovered_address);
        if recovered_address == address {
            Ok(())
        } else {
            Err(BundlrError::InvalidSignature)
        }
    }
}

#[cfg(test)]
mod tests {
    //TODO: implement sign and tests
}
