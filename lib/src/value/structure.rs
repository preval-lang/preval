use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    ir::Module,
    value::{PrevalValue, Value, typ::Type},
    vm::RunResult,
};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
    pub fields: HashMap<String, Option<Value>>,
    pub typ: String,
}
impl PrevalValue for Struct {
    fn get_type(&self) -> Type {
        Type::Struct(self.typ.clone())
    }

    fn vindex(&mut self, value: &Value) -> Value {
        if let Some(name) = value.data.as_any().downcast_ref::<String>() {
            self.fields[name].clone().unwrap()
        } else {
            todo!("Index structs by number")
        }
    }
}
