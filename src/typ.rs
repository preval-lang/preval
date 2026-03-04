use serde::{Deserialize, Serialize};

use crate::{
    expression_parser::{InfoParseError, ParseError},
    tokeniser::{InfoToken, Token, split_by_comma},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    Tuple(Vec<Type>),
    String,
    IO,
    Bool,
    Generic(usize),
    Function(Box<Signature>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Signature {
    pub(crate) args: Vec<Type>,
    pub(crate) returns: Type,
    pub(crate) generics: Vec<()>,
}

pub fn get_type(tokens: &[InfoToken], i: &mut usize) -> Result<Type, InfoParseError> {
    let rv = match &tokens[*i].token {
        Token::Name(name) if name == "String" => Ok(Type::String),
        Token::Name(name) if name == "IO" => Ok(Type::IO),
        Token::Name(name) if name == "Bool" => Ok(Type::Bool),
        Token::Parens(contents) => Ok(Type::Tuple(
            split_by_comma(contents.clone())
                .iter()
                .map(|item| get_type(item, i).unwrap())
                .collect(),
        )),
        _ => Err(InfoParseError {
            idx: tokens[*i].idx,
            error: ParseError::TypeUndefined(tokens.to_vec()),
        }),
    };
    *i += 1;
    rv
}
