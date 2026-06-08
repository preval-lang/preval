use serde::{Deserialize, Serialize};

use crate::ir::{Function, Partial};
use crate::value::native::NativeFunction;
use crate::value::primitive::{EmptyTuple, IO};
use crate::value::structure::Struct;
use crate::value::{PrevalValue, ValueData};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq)]
pub enum TypeDeserializer {
	USize,
	Tuple(Vec<TypeDeserializer>),
	IO,
	Bool,
	String,
	Struct,
	Function,
	Partial,
	NativeFunction,
	Poison,
}

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Poison;

impl PrevalValue for Poison {
	fn get_type(&self) -> TypeDeserializer {
		TypeDeserializer::Poison
	}
}

pub fn deserialize_type(typ: &TypeDeserializer, data: String) -> Box<dyn ValueData> {
	match typ {
		TypeDeserializer::Poison => Box::new(Poison),
		TypeDeserializer::NativeFunction => {
			Box::new(ron::de::from_str::<NativeFunction>(&data).unwrap())
		}
		TypeDeserializer::String => Box::new(ron::de::from_str::<String>(&data).unwrap()),
		TypeDeserializer::Tuple(_) => Box::new(ron::de::from_str::<EmptyTuple>(&data).unwrap()),
		TypeDeserializer::IO => Box::new(IO),
		TypeDeserializer::Bool => Box::new(ron::de::from_str::<bool>(&data).unwrap()),
		TypeDeserializer::USize => Box::new(ron::de::from_str::<usize>(&data).unwrap()),
		TypeDeserializer::Struct => Box::new(ron::de::from_str::<Struct>(&data).unwrap()),
		TypeDeserializer::Function => Box::new(ron::de::from_str::<Function>(&data).unwrap()),
		TypeDeserializer::Partial => Box::new(ron::de::from_str::<Partial>(&data).unwrap()),
	}
}
