use logos::Logos;

use crate::utils::eip712::{error::Eip712Error, lexer::Token};

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Address,
    Uint,
    Int,
    String,
    Bool,
    Bytes,
    Byte(u8),
    Custom(String),
    Array {
        length: Option<u64>,
        inner: Box<Type>,
    },
}

impl ToString for Type {
    fn to_string(&self) -> String {
        match self {
            Type::Address => "address".to_owned(),
            Type::Uint => "uint".to_owned(),
            Type::Int => "int".to_owned(),
            Type::String => "string".to_owned(),
            Type::Bool => "bool".to_owned(),
            Type::Bytes => "bytes".to_owned(),
            Type::Byte(len) => format!("bytes{}", len),
            Type::Custom(custom) => custom.to_string(),
            Type::Array { inner, length } => {
                let inner: String = (*inner).to_string();
                match length {
                    None => format!("{}[]", inner),
                    Some(length) => format!("{}[{}]", inner, length),
                }
            }
        }
    }
}

/// the type string is being validated before it's parsed.
pub fn parse_type(field_type: &str) -> Result<Type, Eip712Error> {
    #[derive(PartialEq)]
    enum State {
        Open,
        Close,
    }

    let mut lexer = Token::lexer(field_type);
    let mut token = None;
    let mut state = State::Close;
    let mut array_depth = 0;
    let mut current_array_length: Option<u64> = None;

    while let Some(result) = lexer.next() {
        if let Ok(lex_token) = result {
            let type_: Type = match lex_token {
                Token::Identifier => Type::Custom(lexer.slice().to_owned()),
                Token::TypeByte => Type::Byte(lexer.extras.0),
                Token::TypeBytes => Type::Bytes,
                Token::TypeBool => Type::Bool,
                Token::TypeUint => Type::Uint,
                Token::TypeInt => Type::Int,
                Token::TypeString => Type::String,
                Token::TypeAddress => Type::Address,
                Token::LiteralInteger => {
                    let length = lexer.slice();
                    current_array_length = Some(
                        length
                            .parse()
                            .map_err(|_| Eip712Error::InvalidArraySize(length.into()))?,
                    );
                    continue;
                }
                Token::BracketOpen if state == State::Close => {
                    state = State::Open;
                    continue;
                }
                Token::BracketClose if array_depth < 10 => {
                    if state == State::Open {
                        let length = current_array_length.take();
                        state = State::Close;
                        token = Some(Type::Array {
                            inner: Box::new(token.expect("if statement checks for some; qed")),
                            length,
                        });
                        array_depth += 1;
                        continue;
                    } else {
                        return Err(Eip712Error::UnexpectedToken(
                            lexer.slice().to_owned(),
                            field_type.to_owned(),
                        ))?;
                    }
                }
                Token::BracketClose if array_depth == 10 => {
                    return Err(Eip712Error::UnsupportedArrayDepth)?;
                }
                _ => {
                    return Err(Eip712Error::UnexpectedToken(
                        lexer.slice().to_owned(),
                        field_type.to_owned(),
                    ))?
                }
            };
            token = Some(type_);
        }
    }

    token.ok_or(Eip712Error::NonExistentType)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let source = "bytes1[][][7][][][][][][][]";
        let ok = parse_type(source).unwrap();
        println!("{:?}", ok);
        assert!(
            ok == Type::Array {
                length: None,
                inner: Box::new(Type::Array {
                    length: None,
                    inner: Box::new(Type::Array {
                        length: None,
                        inner: Box::new(Type::Array {
                            length: None,
                            inner: Box::new(Type::Array {
                                length: None,
                                inner: Box::new(Type::Array {
                                    length: None,
                                    inner: Box::new(Type::Array {
                                        length: None,
                                        inner: Box::new(Type::Array {
                                            length: Some(7),
                                            inner: Box::new(Type::Array {
                                                length: None,
                                                inner: Box::new(Type::Array {
                                                    length: None,
                                                    inner: Box::new(Type::Byte(1))
                                                })
                                            })
                                        })
                                    })
                                })
                            })
                        })
                    })
                })
            }
        );
    }

    #[test]
    fn test_nested_array() {
        let source = "bytes1[][][7][][][][][][][][]";
        assert!(parse_type(source).is_err());
        let source = "byte1[][][7][][][][][][][][]";
        assert!(parse_type(source).is_err());
        let source = "byte[][][7][][][][][][][][]";
        assert!(parse_type(source).is_err());
    }

    #[test]
    fn test_malformed_array_type() {
        let source = "bytes1[7[]uint][]";
        assert!(parse_type(source).is_err());
        let source = "byte[7[]uint][]";
        assert!(parse_type(source).is_err());
    }
}
