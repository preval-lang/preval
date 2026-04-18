use serde::{Deserialize, Serialize};

use crate::ir::{Function, Module, Partial};
use crate::value::native::NativeFunction;
use crate::value::primitive::{EmptyTuple, IO};
use crate::value::structure::Struct;
use crate::value::{PreSerialize, PrevalValue, ValueData};
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
    Partial,
    NativeFunction,
    Poison,
}

impl Type {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Signature {
    pub(crate) args: Vec<Type>,
    pub(crate) returns: Type,
}

pub fn get_type(
    module: &Module,
    tokens: &[InfoToken],
    i: &mut usize,
) -> Result<Type, InfoParseError> {
    let rv = match &tokens[*i].token {
        Token::Name(name) if name == "String" => Ok(Type::String),
        Token::Name(name) if name == "IO" => Ok(Type::IO),
        Token::Name(name) if name == "bool" => Ok(Type::Bool),
        Token::Name(name) if module.structs.contains_key(name) => Ok(Type::Struct(name.clone())),
        _ => Err(InfoParseError {
            idx: tokens[*i].idx,
            error: ParseError::TypeUndefined(tokens.to_vec()),
        }),
    };
    *i += 1;
    rv
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Poison;

impl PrevalValue for Poison {
    fn get_type(&self) -> Type {
        Type::Poison
    }
}

pub fn deserialize_type(typ: &Type, data: String) -> Box<dyn ValueData> {
    match typ {
        Type::Poison => Box::new(Poison),
        Type::NativeFunction => Box::new(ron::de::from_str::<NativeFunction>(&data).unwrap()),
        Type::String => Box::new(ron::de::from_str::<String>(&data).unwrap()),
        Type::Tuple(_) => Box::new(ron::de::from_str::<EmptyTuple>(&data).unwrap()),
        Type::IO => Box::new(IO),
        Type::Bool => Box::new(ron::de::from_str::<bool>(&data).unwrap()),
        Type::USize => Box::new(ron::de::from_str::<usize>(&data).unwrap()),
        Type::Struct(_) => Box::new(ron::de::from_str::<Struct>(&data).unwrap()),
        Type::Function(f) => Box::new(ron::de::from_str::<Function>(&data).unwrap()),
        Type::Partial => Box::new(ron::de::from_str::<Partial>(&data).unwrap()),
    }
}
