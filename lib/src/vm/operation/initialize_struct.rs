use crate::ir::Module;
use crate::value::Value;
use crate::value::structure::Struct;
use crate::value::typ::Type;
use crate::vm::Statement;
use std::collections::HashMap;

pub fn initialize_struct(
    name: String,
    fields: HashMap<String, usize>,
    store: Option<usize>,
    module: &Module,
    out: &mut Vec<Statement>,
    vars: &mut HashMap<usize, Option<Value>>,
) {
    if let Some(store) = store {
        let mut output_struct: HashMap<String, Option<Value>> = HashMap::new();

        let mut residualise = false;

        for (field_name, field_value) in &fields {
            let value = vars.get(field_value).unwrap_or(&None).clone();
            if value.is_none() {
                residualise = true;
            }
            output_struct.insert(field_name.clone(), value);
        }

        vars.insert(
            store,
            Some(Value::new(Struct {
                fields: output_struct,
                typ: name.clone(),
            })),
        );

        if residualise {
            out.push(Statement::Operation(
                crate::ir::Operation::InitializeStruct(name, fields),
                Some(store),
            ));
        }
    }
}
