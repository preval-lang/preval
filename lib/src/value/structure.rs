use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::value::{PrevalValue, Value, runtime_type::TypeDeserializer};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
    pub fields: HashMap<String, Option<Value>>,
}
impl PrevalValue for Struct {
    fn get_type(&self) -> TypeDeserializer {
        TypeDeserializer::Struct
    }

    fn vindex(&mut self, value: &Value) -> Value {
        if let Some(name) = value.data.as_any().downcast_ref::<String>() {
            self.fields[name].clone().unwrap()
        } else {
            todo!("Index structs by number")
        }
    }
}
