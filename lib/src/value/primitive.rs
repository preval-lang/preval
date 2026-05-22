use serde::{Deserialize, Serialize};

use crate::value::{PrevalValue, Value, typ::Type};

impl PrevalValue for String {
    fn vindex(&mut self, value: &Value) -> Value {
        match value.data.as_any().downcast_ref::<usize>() {
            Some(other) => Value::new(self.chars().nth(*other).unwrap().to_string()),
            None => panic!("Index string with non-usize"),
        }
    }

    fn get_type(&self) -> Type {
        Type::String
    }
}

impl PrevalValue for usize {
    fn get_type(&self) -> Type {
        Type::USize
    }
}

impl PrevalValue for bool {
    fn get_type(&self) -> Type {
        Type::Bool
    }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct IO;
impl PrevalValue for IO {
    fn get_type(&self) -> Type {
        Type::IO
    }

    fn vshould_poison(&self) -> bool {
        true
    }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct EmptyTuple;
impl PrevalValue for EmptyTuple {
    fn get_type(&self) -> Type {
        Type::Tuple
    }
}
