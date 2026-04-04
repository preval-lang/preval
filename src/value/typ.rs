use serde::{Deserialize, Serialize};

use crate::ir::{Function, Partial};
use crate::value::ValueData;
use crate::value::native::{Library, LibraryConstructor, Symbol};
use crate::value::primitive::{EmptyTuple, IO};
use crate::value::structure::{Struct, StructConstructor};
use crate::{
    parser::expression::{InfoParseError, ParseError},
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
    Library,
    Symbol,
    LibraryConstructor,
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

pub fn deserialize_type(typ: &Type, data: String) -> Box<dyn ValueData> {
    match typ {
        Type::Symbol => Box::new(ron::de::from_str::<Symbol>(&data).unwrap()),
        Type::LibraryConstructor => {
            Box::new(ron::de::from_str::<LibraryConstructor>(&data).unwrap())
        }
        Type::Library => Box::new(ron::de::from_str::<Library>(&data).unwrap()),
        Type::String => Box::new(ron::de::from_str::<String>(&data).unwrap()),
        Type::Tuple(_) => Box::new(ron::de::from_str::<EmptyTuple>(&data).unwrap()),
        Type::IO => Box::new(ron::de::from_str::<IO>(&data).unwrap()),
        Type::Bool => Box::new(ron::de::from_str::<bool>(&data).unwrap()),
        Type::USize => Box::new(ron::de::from_str::<usize>(&data).unwrap()),
        Type::Struct(_) => Box::new(ron::de::from_str::<Struct>(&data).unwrap()),
        Type::Function(f) => Box::new(ron::de::from_str::<Function>(&data).unwrap()),
        Type::Partial => Box::new(ron::de::from_str::<Partial>(&data).unwrap()),
        Type::StructConstructor(f) => {
            Box::new(ron::de::from_str::<StructConstructor>(&data).unwrap())
        }
    }
}
