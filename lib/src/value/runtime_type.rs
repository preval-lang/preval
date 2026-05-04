use serde::{Deserialize, Serialize};

use crate::ir::{Function, Partial};
use crate::value::native::NativeFunction;
use crate::value::primitive::{EmptyTuple, IO};
use crate::value::structure::Struct;
use crate::value::{PrevalValue, ValueData};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq)]
pub enum RuntimeType {
    USize,
    Tuple(Vec<RuntimeType>),
    IO,
    Bool,
    String,
    Struct(String),
    Function,
    Partial,
    NativeFunction,
    Poison,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Poison;

impl PrevalValue for Poison {
    fn get_type(&self) -> RuntimeType {
        RuntimeType::Poison
    }
}

pub fn deserialize_type(typ: &RuntimeType, data: String) -> Box<dyn ValueData> {
    match typ {
        RuntimeType::Poison => Box::new(Poison),
        RuntimeType::NativeFunction => {
            Box::new(ron::de::from_str::<NativeFunction>(&data).unwrap())
        }
        RuntimeType::String => Box::new(ron::de::from_str::<String>(&data).unwrap()),
        RuntimeType::Tuple(_) => Box::new(ron::de::from_str::<EmptyTuple>(&data).unwrap()),
        RuntimeType::IO => Box::new(IO),
        RuntimeType::Bool => Box::new(ron::de::from_str::<bool>(&data).unwrap()),
        RuntimeType::USize => Box::new(ron::de::from_str::<usize>(&data).unwrap()),
        RuntimeType::Struct(_) => Box::new(ron::de::from_str::<Struct>(&data).unwrap()),
        RuntimeType::Function => Box::new(ron::de::from_str::<Function>(&data).unwrap()),
        RuntimeType::Partial => Box::new(ron::de::from_str::<Partial>(&data).unwrap()),
    }
}
