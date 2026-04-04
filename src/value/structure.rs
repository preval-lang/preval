use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    ir::Module,
    value::{PrevalValue, Value, typ::Type},
    vm::RunResult,
};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
    fields: HashMap<String, Value>,
    typ: String,
}
impl PrevalValue for Struct {
    fn get_type(&self) -> Type {
        Type::Struct(self.typ.clone())
    }

    fn vindex(&mut self, value: &Value) -> Value {
        if let Some(name) = value.data.as_any().downcast_ref::<String>() {
            self.fields[name].clone()
        } else {
            todo!("Index structs by number")
        }
    }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct StructConstructor {
    pub typ: String,
}

impl PrevalValue for StructConstructor {
    fn get_type(&self) -> Type {
        Type::StructConstructor(self.typ.clone())
    }

    fn vcall(&mut self, module: &Module, args: Vec<&Option<Value>>) -> RunResult {
        let st = &module.structs[&self.typ];
        let field_names: Vec<_> = st.fields.keys().collect();
        let mut v = HashMap::new();
        for i in 0..field_names.len() {
            if let Some(arg) = args[i] {
                v.insert(field_names[i].clone(), arg.clone());
            } else {
                return RunResult::Residualise;
            }
        }
        RunResult::Concrete(Value::new(Struct {
            typ: self.typ.clone(),
            fields: v,
        }))
    }
}
