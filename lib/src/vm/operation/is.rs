use std::collections::HashMap;

use crate::{
    ir::{Module, Operation, Statement},
    typ::{TypeReference, type_id},
    value::Value,
};

pub fn is(
    value: usize,
    typ: TypeReference,
    module: &mut Module,
    vars: &mut HashMap<usize, Option<Value>>,
    out: &mut Vec<Statement>,
    store: Option<usize>,
) {
    if let Some(store) = store {
        if let Some(value) = &vars[&value] {
            if module
                .instantiator
                .compatible(TypeReference::Concrete(value.typ), typ, 0)
            {
                vars.insert(store, Some(Value::new(true, type_id::bool)));
            } else {
                vars.insert(store, Some(Value::new(false, type_id::bool)));
            }
        } else {
            vars.insert(store, None);
            out.push(Statement {
                store: Some(store),
                operation: Operation::Is { value, typ },
            });
        }
    }
}
