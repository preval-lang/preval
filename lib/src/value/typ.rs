use serde::{Deserialize, Serialize};

use crate::ir::{Function, Module, Partial};
use crate::value::native::NativeFunction;
use crate::value::primitive::{EmptyTuple, IO};
use crate::value::structure::Struct;
use crate::value::{PrevalValue, ValueData};
use crate::{
    parser::expression::{InfoParseError, ParseError},
    tokeniser::{InfoToken, Token},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq)]
pub enum Type {
    USize,
    Tuple,
    IO,
    Bool,
    String,
    Struct,
    Function,
    Partial,
    NativeFunction,
    Poison,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq)]
pub struct Signature {
    pub(crate) args: Vec<Type>,
    pub(crate) returns: Type,
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
        Type::Tuple => Box::new(ron::de::from_str::<EmptyTuple>(&data).unwrap()),
        Type::IO => Box::new(IO),
        Type::Bool => Box::new(ron::de::from_str::<bool>(&data).unwrap()),
        Type::USize => Box::new(ron::de::from_str::<usize>(&data).unwrap()),
        Type::Struct => Box::new(ron::de::from_str::<Struct>(&data).unwrap()),
        Type::Function => Box::new(ron::de::from_str::<Function>(&data).unwrap()),
        Type::Partial => Box::new(ron::de::from_str::<Partial>(&data).unwrap()),
    }
}
