use super::error::serde_error;
use super::error::Eip712Error;
use super::parser::parse_type;
use super::parser::Type;
use super::MessageTypes;
use super::EIP712;
use indexmap::IndexSet;
use rustc_hex::FromHex;
use serde_json::to_value;
use serde_json::Value;
use std::collections::HashSet;
use std::str::FromStr;
use validator::Validate;
use web3::ethabi::ethereum_types::{Address as EthAddress, H256, U256};
use web3::ethabi::{encode, Token as EthAbiToken};
use web3::signing::keccak256;

fn check_hex(string: &str) -> Result<(), Eip712Error> {
    if string.len() >= 2 && &string[..2] == "0x" {
        return Ok(());
    }

    Err(Eip712Error::HexParseError(format!(
        "Expected a 0x-prefixed string of even length, found {} length string",
        string.len()
    )))
}
/// given a type and HashMap<String, Vec<FieldType>>
/// returns a HashSet of dependent types of the given type
fn build_dependencies<'a>(
    message_type: &'a str,
    message_types: &'a MessageTypes,
) -> Option<HashSet<&'a str>> {
    message_types.get(message_type)?;

    let mut types = IndexSet::new();
    types.insert(message_type);
    let mut deps = HashSet::new();

    while let Some(item) = types.pop() {
        if let Some(fields) = message_types.get(item) {
            deps.insert(item);

            for field in fields {
                // check if this field is an array type
                let field_type = if let Some(index) = field.type_.find('[') {
                    &field.type_[..index]
                } else {
                    &field.type_
                };
                // seen this type before? or not a custom type skip
                if !deps.contains(field_type) || message_types.contains_key(field_type) {
                    types.insert(field_type);
                }
            }
        }
    }

    Some(deps)
}

fn encode_type(message_type: &str, message_types: &MessageTypes) -> Result<String, Eip712Error> {
    let deps = {
        let mut temp =
            build_dependencies(message_type, message_types).ok_or(Eip712Error::NonExistentType)?;
        temp.remove(message_type);
        let mut temp = temp.into_iter().collect::<Vec<_>>();
        (temp[..]).sort_unstable();
        temp.insert(0, message_type);
        temp
    };

    let encoded = deps
        .into_iter()
        .filter_map(|dep| {
            message_types.get(dep).map(|field_types| {
                let types = field_types
                    .iter()
                    .map(|value| format!("{} {}", value.type_, value.name))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("{}({})", dep, types)
            })
        })
        .collect::<Vec<_>>()
        .concat();
    Ok(encoded)
}

fn type_hash(message_type: &str, typed_data: &MessageTypes) -> Result<H256, Eip712Error> {
    let binding = encode_type(message_type, typed_data)?;
    let encoded = binding.as_bytes();
    Ok(web3::types::H256(keccak256(encoded)))
}

fn encode_data(
    message_type: &Type,
    message_types: &MessageTypes,
    value: &Value,
    field_name: Option<&str>,
) -> Result<Vec<u8>, Eip712Error> {
    let encoded = match message_type {
        Type::Array { inner, length } => {
            let mut items = vec![];
            let values = value.as_array().ok_or(serde_error("array", field_name))?;

            // check if the type definition actually matches
            // the length of items to be encoded
            if length.is_some() && Some(values.len() as u64) != *length {
                let array_type = format!("{}[{}]", inner.to_string(), length.unwrap());
                return Err(Eip712Error::UnequalArrayItems(
                    length.unwrap(),
                    array_type,
                    values.len() as u64,
                ))?;
            }

            for item in values {
                let mut encoded = encode_data(inner, message_types, item, field_name)?;
                items.append(&mut encoded);
            }

            keccak256(&items).as_ref().to_vec()
        }

        Type::Custom(ref ident) if message_types.get(ident).is_some() => {
            let type_hash = (type_hash(ident, message_types)?).0.to_vec();
            let mut tokens = encode(&[EthAbiToken::FixedBytes(type_hash)]);

            for field in message_types
                .get(ident)
                .expect("Already checked in match guard; qed")
            {
                let value = &value[&field.name];
                let type_ = parse_type(&field.type_)?;
                let mut encoded = encode_data(&type_, message_types, value, Some(&*field.name))?;
                tokens.append(&mut encoded);
            }

            keccak256(&tokens).as_ref().to_vec()
        }

        Type::Bytes => {
            let string = value.as_str().ok_or(serde_error("string", field_name))?;

            check_hex(string)?;

            let bytes = (string[2..])
                .from_hex::<Vec<u8>>()
                .map_err(|err| Eip712Error::HexParseError(format!("{}", err)))?;
            let bytes = keccak256(&bytes).as_ref().to_vec();

            encode(&[EthAbiToken::FixedBytes(bytes)])
        }

        Type::Byte(_) => {
            let string = value.as_str().ok_or(serde_error("string", field_name))?;

            check_hex(string)?;

            let bytes = (string[2..])
                .from_hex::<Vec<u8>>()
                .map_err(|err| Eip712Error::HexParseError(format!("{}", err)))?;

            encode(&[EthAbiToken::FixedBytes(bytes)])
        }

        Type::String => {
            let value = value.as_str().ok_or(serde_error("string", field_name))?;
            let hash = keccak256(value.as_bytes()).as_ref().to_vec();
            encode(&[EthAbiToken::FixedBytes(hash)])
        }

        Type::Bool => encode(&[EthAbiToken::Bool(
            value.as_bool().ok_or(serde_error("bool", field_name))?,
        )]),

        Type::Address => {
            let addr = value.as_str().ok_or(serde_error("string", field_name))?;
            if addr.len() != 42 {
                return Err(Eip712Error::InvalidAddressLength(addr.len()))?;
            }
            let address = EthAddress::from_str(&addr[2..])
                .map_err(|err| Eip712Error::HexParseError(format!("{}", err)))?;
            encode(&[EthAbiToken::Address(address)])
        }

        Type::Uint | Type::Int => {
            let string = value.as_str().ok_or(serde_error("int/uint", field_name))?;

            check_hex(string)?;

            let uint = U256::from_str(&string[2..])
                .map_err(|err| Eip712Error::HexParseError(format!("{}", err)))?;

            let token = if *message_type == Type::Uint {
                EthAbiToken::Uint(uint)
            } else {
                EthAbiToken::Int(uint)
            };
            encode(&[token])
        }

        _ => {
            return Err(Eip712Error::UnknownType(
                field_name.unwrap_or("").to_string(),
                message_type.to_string(),
            ));
        }
    };

    Ok(encoded)
}

/// encodes and hashes the given EIP712 struct
pub fn hash_structured_data(typed_data: EIP712) -> Result<[u8; 32], Eip712Error> {
    // validate input
    typed_data
        .validate()
        .map_err(Eip712Error::ValidationErrors)?;
    // EIP-191 compliant
    let prefix = (b"\x19\x01").to_vec();
    let domain = to_value(&typed_data.domain).unwrap();
    let (domain_hash, data_hash) = (
        encode_data(
            &Type::Custom("EIP712Domain".into()),
            &typed_data.types,
            &domain,
            None,
        )?,
        encode_data(
            &Type::Custom(typed_data.primary_type),
            &typed_data.types,
            &typed_data.message,
            None,
        )?,
    );
    let concat = [&prefix[..], &domain_hash[..], &data_hash[..]].concat();
    Ok(keccak256(&concat))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustc_hex::ToHex;
    use serde_json::from_str;

    const JSON: &'static str = r#"{
		"primaryType": "Mail",
		"domain": {
			"name": "Ether Mail",
			"version": "1",
			"chainId": "0x1",
			"verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
		},
		"message": {
			"from": {
				"name": "Cow",
				"wallet": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
			},
			"to": {
				"name": "Bob",
				"wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
			},
			"contents": "Hello, Bob!"
		},
		"types": {
			"EIP712Domain": [
				{ "name": "name", "type": "string" },
				{ "name": "version", "type": "string" },
				{ "name": "chainId", "type": "uint256" },
				{ "name": "verifyingContract", "type": "address" }
			],
			"Person": [
				{ "name": "name", "type": "string" },
				{ "name": "wallet", "type": "address" }
			],
			"Mail": [
				{ "name": "from", "type": "Person" },
				{ "name": "to", "type": "Person" },
				{ "name": "contents", "type": "string" }
			]
		}
	}"#;

    #[test]
    fn test_build_dependencies() {
        let string = r#"{
			"EIP712Domain": [
				{ "name": "name", "type": "string" },
				{ "name": "version", "type": "string" },
				{ "name": "chainId", "type": "uint256" },
				{ "name": "verifyingContract", "type": "address" }
			],
			"Person": [
				{ "name": "name", "type": "string" },
				{ "name": "wallet", "type": "address" }
			],
			"Mail": [
				{ "name": "from", "type": "Person" },
				{ "name": "to", "type": "Person" },
				{ "name": "contents", "type": "string" }
			]
		}"#;

        let value = from_str::<MessageTypes>(string).expect("alas error!");
        let mail = "Mail";
        let person = "Person";

        let hashset = {
            let mut temp = HashSet::new();
            temp.insert(mail);
            temp.insert(person);
            temp
        };
        assert_eq!(build_dependencies(mail, &value), Some(hashset));
    }

    #[test]
    fn test_encode_type() {
        let string = r#"{
			"EIP712Domain": [
				{ "name": "name", "type": "string" },
				{ "name": "version", "type": "string" },
				{ "name": "chainId", "type": "uint256" },
				{ "name": "verifyingContract", "type": "address" }
			],
			"Person": [
				{ "name": "name", "type": "string" },
				{ "name": "wallet", "type": "address" }
			],
			"Mail": [
				{ "name": "from", "type": "Person" },
				{ "name": "to", "type": "Person" },
				{ "name": "contents", "type": "string" }
			]
		}"#;

        let value = from_str::<MessageTypes>(string).expect("alas error!");
        let mail = &String::from("Mail");
        assert_eq!(
            "Mail(Person from,Person to,string contents)Person(string name,address wallet)",
            encode_type(&mail, &value).expect("alas error!")
        )
    }

    #[test]
    fn test_encode_type_hash() {
        let string = r#"{
			"EIP712Domain": [
				{ "name": "name", "type": "string" },
				{ "name": "version", "type": "string" },
				{ "name": "chainId", "type": "uint256" },
				{ "name": "verifyingContract", "type": "address" }
			],
			"Person": [
				{ "name": "name", "type": "string" },
				{ "name": "wallet", "type": "address" }
			],
			"Mail": [
				{ "name": "from", "type": "Person" },
				{ "name": "to", "type": "Person" },
				{ "name": "contents", "type": "string" }
			]
		}"#;

        let value = from_str::<MessageTypes>(string).expect("alas error!");
        let mail = &String::from("Mail");
        let hash = (type_hash(&mail, &value).expect("alas error!").0).to_hex::<String>();
        assert_eq!(
            hash,
            "a0cedeb2dc280ba39b857546d74f5549c3a1d7bdc2dd96bf881f76108e23dac2"
        );
    }

    #[test]
    fn test_hash_data() {
        let typed_data = from_str::<EIP712>(JSON).expect("alas error!");
        let hash = hash_structured_data(typed_data).expect("alas error!");
        assert_eq!(
            &format!("{:x}", web3::types::H256(hash))[..],
            "be609aee343fb3c4b28e1df9e632fca64fcfaede20f02e86244efddf30957bd2",
        )
    }

    #[test]
    fn test_unequal_array_lengths() {
        const TEST: &'static str = r#"{
		"primaryType": "Mail",
		"domain": {
			"name": "Ether Mail",
			"version": "1",
			"chainId": "0x1",
			"verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
		},
		"message": {
			"from": {
				"name": "Cow",
				"wallet": "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826"
			},
			"to": [{
				"name": "Bob",
				"wallet": "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB"
			}],
			"contents": "Hello, Bob!"
		},
		"types": {
			"EIP712Domain": [
				{ "name": "name", "type": "string" },
				{ "name": "version", "type": "string" },
				{ "name": "chainId", "type": "uint256" },
				{ "name": "verifyingContract", "type": "address" }
			],
			"Person": [
				{ "name": "name", "type": "string" },
				{ "name": "wallet", "type": "address" }
			],
			"Mail": [
				{ "name": "from", "type": "Person" },
				{ "name": "to", "type": "Person[2]" },
				{ "name": "contents", "type": "string" }
			]
		}
	}"#;

        let typed_data = from_str::<EIP712>(TEST).expect("alas error!");
        assert_eq!(
            hash_structured_data(typed_data).unwrap_err(),
            Eip712Error::UnequalArrayItems(2, "Person[2]".into(), 1)
        )
    }

    #[test]
    fn test_typed_data_v4() {
        let string = r#"{
            "types": {
                "EIP712Domain": [
                    {
                      "name": "name",
                      "type": "string"
                    },
                    {
                      "name": "version",
                      "type": "string"
                    },
                    {
                      "name": "chainId",
                      "type": "uint256"
                    },
                    {
                      "name": "verifyingContract",
                      "type": "address"
                    }
                ],
                "Person": [
                    {
                      "name": "name",
                      "type": "string"
                    },
                    {
                      "name": "wallets",
                      "type": "address[]"
                    }
                ],
                "Mail": [
                    {
                      "name": "from",
                      "type": "Person"
                    },
                    {
                      "name": "to",
                      "type": "Person[]"
                    },
                    {
                      "name": "contents",
                      "type": "string"
                    }
                ],
                "Group": [
                    {
                      "name": "name",
                      "type": "string"
                    },
                    {
                      "name": "members",
                      "type": "Person[]"
                    }
                ]
            },
            "domain": {
                "name": "Ether Mail",
                "version": "1",
                "chainId": "0x1",
                "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
            },
            "primaryType": "Mail",
            "message": {
                "from": {
                    "name": "Cow",
                    "wallets": [
                      "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826",
                      "0xDeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF"
                    ]
                },
                "to": [
                    {
                        "name": "Bob",
                        "wallets": [
                            "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB",
                            "0xB0BdaBea57B0BDABeA57b0bdABEA57b0BDabEa57",
                            "0xB0B0b0b0b0b0B000000000000000000000000000"
                        ]
                    }
                ],
                "contents": "Hello, Bob!"
            }
        }"#;

        let typed_data = from_str::<EIP712>(string).expect("alas error!");
        let hash = hash_structured_data(typed_data.clone()).expect("alas error!");
        assert_eq!(
            &format!("{:x}", web3::types::H256(hash))[..],
            "a85c2e2b118698e88db68a8105b794a8cc7cec074e89ef991cb4f5f533819cc2",
        );
    }

    #[test]
    fn test_typed_data_v4_custom_array() {
        let string = r#"{
            "types": {
                "EIP712Domain": [
                    {
                        "name": "name",
                        "type": "string"
                    },
                    {
                        "name": "version",
                        "type": "string"
                    },
                    {
                        "name": "chainId",
                        "type": "uint256"
                    },
                    {
                        "name": "verifyingContract",
                        "type": "address"
                    }
                ],
              "Person": [
                {
                  "name": "name",
                  "type": "string"
                },
                {
                  "name": "wallets",
                  "type": "address[]"
                }
              ],
              "Mail": [
                {
                  "name": "from",
                  "type": "Person"
                },
                {
                  "name": "to",
                  "type": "Group"
                },
                {
                  "name": "contents",
                  "type": "string"
                }
              ],
              "Group": [
                {
                  "name": "name",
                  "type": "string"
                },
                {
                  "name": "members",
                  "type": "Person[]"
                }
              ]
            },
            "domain": {
              "name": "Ether Mail",
              "version": "1",
              "chainId": "0x1",
              "verifyingContract": "0xCcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"
            },
            "primaryType": "Mail",
            "message": {
              "from": {
                "name": "Cow",
                "wallets": [
                  "0xCD2a3d9F938E13CD947Ec05AbC7FE734Df8DD826",
                  "0xDeaDbeefdEAdbeefdEadbEEFdeadbeEFdEaDbeeF"
                ]
              },
              "to": {
                "name": "Farmers",
                "members": [
                  {
                    "name": "Bob",
                    "wallets": [
                      "0xbBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB",
                      "0xB0BdaBea57B0BDABeA57b0bdABEA57b0BDabEa57",
                      "0xB0B0b0b0b0b0B000000000000000000000000000"
                    ]
                  }
                ]
              },
              "contents": "Hello, Bob!"
            }
          }"#;
        let typed_data = from_str::<EIP712>(string).expect("alas error!");
        let hash = hash_structured_data(typed_data.clone()).expect("alas error!");

        assert_eq!(
            &format!("{:x}", web3::types::H256(hash))[..],
            "cd8b34cd09c541cfc0a2fcd147e47809b98b335649c2aa700db0b0c4501a02a0",
        );
    }
}
