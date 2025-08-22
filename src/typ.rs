use crate::{
    expression_parser::{InfoParseError, ParseError},
    tokeniser::{InfoToken, Token},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    usize,
    void,
    Pointer(Pointer),
    u8,
    Slice(Box<Type>),
    Array(Box<Type>, usize),
    IO,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pointer {
    Function(Box<Signature>),
    Value(Box<Type>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Signature {
    pub(crate) args: Vec<Type>,
    pub(crate) returns: Type,
}

pub fn get_type(tokens: &[InfoToken], i: &mut usize) -> Result<Type, InfoParseError> {
    let rv = match &tokens[*i].token {
        Token::Name(name) if name == "StringSlice" => Ok(Type::Slice(Box::new(Type::u8))),
        Token::Name(name) if name == "IO" => Ok(Type::IO),
        _ => Err(InfoParseError {
            idx: tokens[*i].idx,
            error: ParseError::TypeUndefined(tokens.to_vec()),
        }),
    };
    *i += 1;
    rv
}
