use serde::{Deserialize, Serialize};

use crate::value::{PrevalValue, Value, runtime_type::TypeDeserializer};

impl PrevalValue for String {
	fn vindex(&mut self, _value: &Value) -> Value {
		// match value.data.as_any().downcast_ref::<usize>() {
		//     Some(other) => Value::new(self.chars().nth(*other).unwrap().to_string()),
		//     None => panic!("Index string with non-usize"),
		// }
		todo!("Pass around type context to enable creation of values from rust")
	}

	fn get_type(&self) -> TypeDeserializer {
		TypeDeserializer::String
	}
}

impl PrevalValue for usize {
	fn get_type(&self) -> TypeDeserializer {
		TypeDeserializer::USize
	}
}

impl PrevalValue for bool {
	fn get_type(&self) -> TypeDeserializer {
		TypeDeserializer::Bool
	}
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct IO;
impl PrevalValue for IO {
	fn get_type(&self) -> TypeDeserializer {
		TypeDeserializer::IO
	}

	fn vshould_poison(&self) -> bool {
		true
	}
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct EmptyTuple;
impl PrevalValue for EmptyTuple {
	fn get_type(&self) -> TypeDeserializer {
		TypeDeserializer::Tuple(Vec::new())
	}
}
