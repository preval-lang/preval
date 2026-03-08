use serde::{Deserialize, Serialize};

use crate::{
    expression_parser::{InfoParseError, ParseError},
    tokeniser::{InfoToken, Token},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    USize,
    Tuple(Vec<Type>),
    Uint8,
    IO,
    Bool,
    String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Signature {
    pub(crate) args: Vec<Type>,
    pub(crate) returns: Type,
}

pub fn get_type(tokens: &[InfoToken], i: &mut usize) -> Result<Type, InfoParseError> {
    let rv = match &tokens[*i].token {
        Token::Name(name) if name == "String" => Ok(Type::String),
        Token::Name(name) if name == "IO" => Ok(Type::IO),
        Token::Name(name) if name == "bool" => Ok(Type::Bool),
        _ => Err(InfoParseError {
            idx: tokens[*i].idx,
            error: ParseError::TypeUndefined(tokens.to_vec()),
        }),
    };
    *i += 1;
    rv
}
