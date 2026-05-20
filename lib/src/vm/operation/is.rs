use std::collections::HashMap;

use crate::{
    ir::{Module, Operation, Statement},
    typ::Type,
    value::Value,
};

pub fn is(
    value: usize,
    typ: Type,
    module: &mut Module,
    vars: &mut HashMap<usize, Option<Value>>,
    out: &mut Vec<Statement>,
    store: Option<usize>,
) {
    if let Some(store) = store {
        if let Some(value) = &vars[&value] {
            todo!("Check type compatibility")
        } else {
            vars.insert(store, None);
            out.push(Statement {
                store: Some(store),
                operation: Operation::Is { value, typ },
            });
        }
    }
}
