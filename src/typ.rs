use serde::{Deserialize, Serialize};

use crate::ir::{Function, Partial};
use crate::value::{EmptyTuple, IO, Print, Struct, StructConstructor, ValueData};
use crate::{
    expression_parser::{InfoParseError, ParseError},
    tokeniser::{InfoToken, Token},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    USize,
    Tuple(Vec<Type>),
    IO,
    Bool,
    String,
    Struct(String),
    Function(Box<Signature>),
    StructConstructor(String),
    Partial,
    Print,
}

impl Type {}

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

pub fn deserialize_type(typ: &Type, data: serde_value::Value) -> Box<dyn ValueData> {
    match typ {
        Type::String => Box::new(data.deserialize_into::<String>().unwrap()),
        Type::Tuple(_) => Box::new(data.deserialize_into::<EmptyTuple>().unwrap()),
        Type::IO => Box::new(data.deserialize_into::<IO>().unwrap()),
        Type::Bool => Box::new(data.deserialize_into::<bool>().unwrap()),
        Type::USize => Box::new(data.deserialize_into::<usize>().unwrap()),
        Type::Struct(_) => Box::new(data.deserialize_into::<Struct>().unwrap()),
        Type::Function(f) => Box::new(data.deserialize_into::<Function>().unwrap()),
        Type::Partial => Box::new(data.deserialize_into::<Partial>().unwrap()),
        Type::Print => Box::new(data.deserialize_into::<Print>().unwrap()),
        Type::StructConstructor(f) => {
            Box::new(data.deserialize_into::<StructConstructor>().unwrap())
        }
    }
}
